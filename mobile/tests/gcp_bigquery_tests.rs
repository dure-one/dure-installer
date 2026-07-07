//! Tests for api/gcp/bigquery module

use dure::api::gcp::bigquery::{BigQueryResponse, BillingRecord};

#[test]
fn test_bigquery_response_structure() {
    let response = BigQueryResponse {
        kind: "bigquery#queryResponse".to_string(),
        schema: None,
        rows: None,
        total_rows: Some("0".to_string()),
        job_complete: true,
    };

    assert_eq!(response.kind, "bigquery#queryResponse");
    assert!(response.job_complete);
}

#[test]
fn test_billing_record_structure() {
    let record = BillingRecord {
        month: "2024-01".to_string(),
        currency: "USD".to_string(),
        total_net_cost: 123.45,
    };

    assert_eq!(record.month, "2024-01");
    assert_eq!(record.currency, "USD");
    assert_eq!(record.total_net_cost, 123.45);
}
