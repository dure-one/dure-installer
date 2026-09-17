pub mod about;
pub mod about_stt;
pub mod clipboard_popup;
pub mod settings;
pub mod settings_stt;
pub mod uninstall_confirm;
pub mod uninstall_confirm_stt;
pub mod update;
pub mod update_stt;
// pub mod window;

#[cfg(not(target_arch = "wasm32"))]
pub mod platform_gcp;

pub use about_stt::*;
pub use settings_stt::*;
pub use uninstall_confirm_stt::*;
pub use update_stt::*;
