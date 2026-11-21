use crate::api::gcp::bigquery::BillingRecord;

#[derive(Default)]
pub struct BillingDialog {
    pub open: bool,
    pub project_id: String,
    pub data: Option<Vec<BillingRecord>>,
    pub loading: bool,
    pub error: Option<String>,
    pub dataset: String,
    pub table: String,
}

pub enum BillingAction {
    Refresh,
    Close,
    None,
}
