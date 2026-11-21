#[derive(Default)]
pub struct AddPlatformDialog {
    pub open: bool,
    pub platform_type: String,
    pub oauth_url: Option<String>,
    pub oauth_result: Option<crate::api::gcp::oauth::OAuthResult>,
    pub oauth_promise:
        Option<poll_promise::Promise<Result<crate::api::gcp::oauth::OAuthResult, String>>>,
    pub connected_email: Option<String>,
    pub project_list: Vec<(String, String)>,
    pub selected_project: Option<usize>,
    pub create_new: bool,
    pub new_project_id: String,
    pub reauth_mode: bool,
    pub existing_project_id: String,
}

pub enum AddPlatformAction {
    Add {
        platform_type: String,
        project_id: String,
        oauth_result: crate::api::gcp::oauth::OAuthResult,
    },
    Connect,
    FetchProjects,
    Cancel,
    None,
}
