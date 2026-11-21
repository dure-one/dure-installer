// DNS-related dialogs

pub mod ns_check_dialog_stt;
pub mod ns_check_dialog;
pub use ns_check_dialog_stt::*;
pub use ns_check_dialog::*;

pub mod https_check_dialog_stt;
pub mod https_check_dialog;
pub use https_check_dialog_stt::*;
pub use https_check_dialog::*;

pub mod dns_settings_dialog_stt;
pub mod dns_settings_dialog;
pub use dns_settings_dialog_stt::*;
pub use dns_settings_dialog::*;
