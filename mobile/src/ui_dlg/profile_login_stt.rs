#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Default)]
pub struct DlgProfileLogin {
    pub open: bool,
    pub password: String,
    pub error_message: String,
    pub confirmed: bool,
}
