// Platform-related dialogs
// TODO: Extract dialogs from ui_tabs/platform.rs and platform_gcp.rs

pub mod error_dialog_stt;
pub mod error_dialog;

pub mod delete_firewall_confirm_stt;
pub mod delete_firewall_confirm;

pub mod delete_platform_confirm_stt;
pub mod delete_platform_confirm;

pub mod restart_vm_confirm_stt;
pub mod restart_vm_confirm;

pub mod select_project_dialog_stt;
pub mod select_project_dialog;

pub mod bind_vm_dialog_stt;
pub mod bind_vm_dialog;

pub mod delete_vm_dialog_stt;
pub mod delete_vm_dialog;

pub mod billing_dialog_stt;
pub mod billing_dialog;

pub mod add_firewall_dialog_stt;
pub mod add_firewall_dialog;

pub mod firewall_dialog_stt;
pub mod firewall_dialog;

pub mod add_platform_dialog_stt;
pub mod add_platform_dialog;

pub use error_dialog::*;
pub use error_dialog_stt::*;
pub use delete_firewall_confirm::*;
pub use delete_firewall_confirm_stt::*;
pub use delete_platform_confirm::*;
pub use delete_platform_confirm_stt::*;
pub use restart_vm_confirm::*;
pub use restart_vm_confirm_stt::*;
pub use select_project_dialog::*;
pub use select_project_dialog_stt::*;
pub use bind_vm_dialog::*;
pub use bind_vm_dialog_stt::*;
pub use delete_vm_dialog::*;
pub use delete_vm_dialog_stt::*;
pub use billing_dialog::*;
pub use billing_dialog_stt::*;
pub use add_firewall_dialog::*;
pub use add_firewall_dialog_stt::*;
pub use firewall_dialog::*;
pub use firewall_dialog_stt::*;
pub use add_platform_dialog::*;
pub use add_platform_dialog_stt::*;
