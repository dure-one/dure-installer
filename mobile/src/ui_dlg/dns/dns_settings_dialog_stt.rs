#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct DnsRecordDisplay {
    pub name: String,
    pub record_type: String,
    pub value: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SettingsCheckResult {
    pub checked_at: i64,
    pub records_valid: bool,
    pub missing_records: Vec<String>,
    pub error: Option<String>,
    #[serde(default)]
    pub fetched_records: Vec<DnsRecordDisplay>,
}

#[derive(Default)]
pub struct DnsSettingsDialog {
    pub open: bool,
    pub result: Option<(SettingsCheckResult, Vec<String>)>,
    pub domain: Option<String>,
    pub has_oauth_token: bool,
    pub vm_ip: Option<String>,
    pub operation_log: Vec<String>,
}

pub enum DnsSettingsAction {
    Authenticate { domain: String },
    SetupDns,
    Close,
    None,
}
