#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct HttpsCheckResult {
    pub checked_at: i64,
    pub resolves_to_vm: bool,
    pub resolved_ip: Option<String>,
    pub expected_ip: Option<String>,
    pub error: Option<String>,
}

#[derive(Default)]
pub struct HttpsCheckDialog {
    pub open: bool,
    pub result: Option<HttpsCheckResult>,
}

pub enum HttpsCheckAction {
    Close,
    None,
}
