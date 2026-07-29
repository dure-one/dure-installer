//! GCP BigQuery API module
//!
//! Handles BigQuery datasets, tables, queries, and billing data analysis.

use crate::{dure_info, dure_debug, dure_warn, dure_error};
use anyhow::Result;
use chrono::Datelike;
use serde::{Deserialize, Serialize};

use super::GcpRestClient;

// ============================================================================
// BigQuery Types
// ============================================================================

/// BigQuery API response types
#[derive(Debug, Serialize, Deserialize)]
pub struct BigQueryResponse {
    pub kind: String,
    pub schema: Option<BigQuerySchema>,
    pub rows: Option<Vec<BigQueryRow>>,
    #[serde(rename = "totalRows")]
    pub total_rows: Option<String>,
    #[serde(rename = "jobComplete")]
    pub job_complete: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BigQuerySchema {
    pub fields: Vec<BigQueryField>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BigQueryField {
    pub name: String,
    #[serde(rename = "type")]
    pub field_type: String,
    pub mode: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BigQueryRow {
    pub f: Vec<BigQueryCell>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BigQueryCell {
    pub v: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingRecord {
    pub month: String,
    pub currency: String,
    pub total_net_cost: f64,
}

// ============================================================================
// API Methods on GcpRestClient
// ============================================================================

impl GcpRestClient {
    /// List BigQuery datasets
    pub fn list_bigquery_datasets(&self, project_id: &str) -> Result<Vec<String>> {
        let url = format!(
            "https://bigquery.googleapis.com/bigquery/v2/projects/{}/datasets",
            project_id
        );

        let response = self.get(&url)?;
        let body: serde_json::Value = response.into_json()?;

        let mut datasets = Vec::new();
        if let Some(dataset_list) = body["datasets"].as_array() {
            for dataset in dataset_list {
                if let Some(dataset_id) = dataset["datasetReference"]["datasetId"].as_str() {
                    datasets.push(dataset_id.to_string());
                }
            }
        }
        Ok(datasets)
    }

    /// List BigQuery tables in a dataset
    pub fn list_bigquery_tables(&self, project_id: &str, dataset_id: &str) -> Result<Vec<String>> {
        let url = format!(
            "https://bigquery.googleapis.com/bigquery/v2/projects/{}/datasets/{}/tables",
            project_id, dataset_id
        );

        let response = self.get(&url)?;
        let body: serde_json::Value = response.into_json()?;

        let mut tables = Vec::new();
        if let Some(table_list) = body["tables"].as_array() {
            for table in table_list {
                if let Some(table_id) = table["tableReference"]["tableId"].as_str() {
                    tables.push(table_id.to_string());
                }
            }
        }
        Ok(tables)
    }

    /// Auto-discover billing export table
    pub fn discover_billing_table(&self, project_id: &str) -> Result<(String, String)> {
        // List all datasets
        let datasets = self.list_bigquery_datasets(project_id)?;

        // Look for billing-related datasets
        for dataset in datasets {
            if dataset.contains("billing") || dataset.contains("export") {
                // List tables in this dataset
                if let Ok(tables) = self.list_bigquery_tables(project_id, &dataset) {
                    // Look for gcp_billing_export_v1_* table
                    for table in tables {
                        if table.starts_with("gcp_billing_export_v1_") {
                            return Ok((dataset, table));
                        }
                    }
                }
            }
        }

        Err(anyhow::anyhow!(
            "No billing export table found. Please configure billing export in GCP Console."
        ))
    }

    /// Query BigQuery for billing data
    pub fn query_bigquery(&self, project_id: &str, query: &str) -> Result<BigQueryResponse> {
        let url = format!(
            "https://bigquery.googleapis.com/bigquery/v2/projects/{}/queries",
            project_id
        );

        let request = serde_json::json!({
            "query": query,
            "useLegacySql": false,
            "maxResults": 10000
        });

        let response = self.post(&url, &request.to_string())?;

        // Check for HTTP error status
        let status = response.status();
        let body_text = response.into_string()?;

        if status != 200 {
            // Try to parse error message from response
            if let Ok(error_json) = serde_json::from_str::<serde_json::Value>(&body_text) {
                if let Some(error_msg) = error_json["error"]["message"].as_str() {
                    return Err(anyhow::anyhow!("BigQuery API error: {}", error_msg));
                }
            }
            return Err(anyhow::anyhow!(
                "BigQuery API error (HTTP {}): {}",
                status,
                body_text
            ));
        }

        // Parse successful response
        let result: BigQueryResponse = serde_json::from_str(&body_text)?;
        Ok(result)
    }

    /// Get billing data by month using BigQuery billing export
    ///
    /// This method queries the detailed cost data exported to BigQuery to calculate
    /// monthly total costs including all credits.
    ///
    /// Returns billing data for the last 3 months plus the current month (4 months total).
    ///
    /// To enable billing export:
    /// 1. Go to GCP Console → Billing → Billing export
    /// 2. Select "Detailed cost data" tab
    /// 3. Select or create a BigQuery dataset
    /// 4. Wait a few hours for data to populate
    pub fn get_current_month_billing(
        &self,
        project_id: &str,
        dataset_id: &str,
        table_id: &str,
    ) -> Result<Vec<BillingRecord>> {
        let now = chrono::Utc::now();

        // Calculate date range: 3 months back from start of current month
        let three_months_ago = if now.month() > 3 {
            chrono::NaiveDate::from_ymd_opt(now.year(), now.month() - 3, 1)
                .unwrap_or_else(|| chrono::NaiveDate::from_ymd_opt(now.year(), 1, 1).unwrap())
        } else {
            // Handle year boundary (e.g., if current month is Jan, Feb, or Mar)
            let new_year = now.year() - 1;
            let new_month = (now.month() + 12 - 3) % 12;
            let new_month = if new_month == 0 { 12 } else { new_month };
            chrono::NaiveDate::from_ymd_opt(new_year, new_month, 1)
                .unwrap_or_else(|| chrono::NaiveDate::from_ymd_opt(now.year(), 1, 1).unwrap())
        };

        let start_date = three_months_ago.format("%Y-%m-%d").to_string();
        let end_date = now.format("%Y-%m-%d").to_string();

        // Query for monthly billing totals with currency (last 3 months + current)
        // This retrieves the actual currency from the billing data
        let query = format!(
            r#"
            SELECT
              FORMAT_DATE('%Y-%m', DATE(usage_start_time)) AS month,
              currency,
              ROUND(SUM(cost) + SUM(IFNULL((SELECT SUM(c.amount) FROM UNNEST(credits) c), 0)), 2) AS total_net_cost
            FROM
              `{}.{}.{}`
            WHERE
              DATE(usage_start_time) >= '{}'
              AND DATE(usage_start_time) <= '{}'
              AND cost IS NOT NULL
            GROUP BY
              month, currency
            ORDER BY
              month DESC, currency
            "#,
            project_id, dataset_id, table_id, start_date, end_date
        );

        let response = self.query_bigquery(project_id, &query)?;

        let mut records = Vec::new();
        if let Some(rows) = response.rows {
            for row in rows {
                if row.f.len() >= 3 {
                    let month = row.f[0].v.clone().unwrap_or_default();
                    let currency = row.f[1].v.clone().unwrap_or_else(|| "USD".to_string());
                    let cost_str = row.f[2].v.clone().unwrap_or_else(|| "0.0".to_string());
                    let total_net_cost: f64 = cost_str.parse().unwrap_or(0.0);

                    records.push(BillingRecord {
                        month,
                        currency,
                        total_net_cost,
                    });
                }
            }
        }

        Ok(records)
    }
}
