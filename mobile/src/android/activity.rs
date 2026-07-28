// Android Activity launcher for opening system settings and apps via JNI
// Provides functions to launch specific Android settings screens and apps.

#[cfg(target_os = "android")]
use crate::{dure_info, dure_debug, dure_warn, dure_error};
use jni::objects::JValue;

#[cfg(target_os = "android")]
use ndk_context;

/// Open Device Info settings with build_number highlighted (for enabling Developer Mode)
#[cfg(target_os = "android")]
pub fn open_build_number_settings() {
    if let Err(e) = open_build_number_settings_inner() {
        dure_error!("Failed to open build number settings: {}", e);
    }
}

#[cfg(target_os = "android")]
fn open_build_number_settings_inner() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = ndk_context::android_context();
    let vm = unsafe { jni::JavaVM::from_raw(ctx.vm() as _) }?;
    let activity = unsafe { jni::objects::JObject::from_raw(ctx.context() as _) };
    let mut env = vm.attach_current_thread()?;

    let intent_class = env.find_class("android/content/Intent")?;

    // Try ACTION_DEVICE_INFO_SETTINGS with build_number highlight
    let action = env.new_string("android.settings.DEVICE_INFO_SETTINGS")?;
    let intent = env.new_object(
        &intent_class,
        "(Ljava/lang/String;)V",
        &[JValue::Object(&action)],
    )?;

    // Add extra to highlight build_number
    let extra_key = env.new_string(":settings:fragment_args_key")?;
    let extra_value = env.new_string("build_number")?;
    env.call_method(
        &intent,
        "putExtra",
        "(Ljava/lang/String;Ljava/lang/String;)Landroid/content/Intent;",
        &[JValue::Object(&extra_key), JValue::Object(&extra_value)],
    )?;

    match env.call_method(
        &activity,
        "startActivity",
        "(Landroid/content/Intent;)V",
        &[JValue::Object(&intent)],
    ) {
        Ok(_) => Ok(()),
        Err(_) => {
            // Fallback to general settings
            env.exception_clear()?;
            let fallback_action = env.new_string("android.settings.SETTINGS")?;
            let fallback_intent = env.new_object(
                &intent_class,
                "(Ljava/lang/String;)V",
                &[JValue::Object(&fallback_action)],
            )?;
            env.call_method(
                &activity,
                "startActivity",
                "(Landroid/content/Intent;)V",
                &[JValue::Object(&fallback_intent)],
            )?;
            Ok(())
        }
    }
}

/// Open Developer Options settings with wireless debugging highlighted
#[cfg(target_os = "android")]
pub fn open_wireless_debugging_settings() {
    if let Err(e) = open_wireless_debugging_settings_inner() {
        dure_error!("Failed to open wireless debugging settings: {}", e);
    }
}

#[cfg(target_os = "android")]
fn open_wireless_debugging_settings_inner() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = ndk_context::android_context();
    let vm = unsafe { jni::JavaVM::from_raw(ctx.vm() as _) }?;
    let activity = unsafe { jni::objects::JObject::from_raw(ctx.context() as _) };
    let mut env = vm.attach_current_thread()?;

    let intent_class = env.find_class("android/content/Intent")?;
    let action = env.new_string("android.settings.APPLICATION_DEVELOPMENT_SETTINGS")?;
    let intent = env.new_object(
        &intent_class,
        "(Ljava/lang/String;)V",
        &[JValue::Object(&action)],
    )?;

    // Set flags: FLAG_ACTIVITY_NEW_TASK | FLAG_ACTIVITY_CLEAR_TASK
    let flags: i32 = 0x10000000 | 0x00008000;
    env.call_method(
        &intent,
        "setFlags",
        "(I)Landroid/content/Intent;",
        &[JValue::Int(flags)],
    )?;

    // Add extra to highlight wireless debugging toggle
    let extra_key = env.new_string(":settings:fragment_args_key")?;
    let extra_value = env.new_string("toggle_adb_wireless")?;
    env.call_method(
        &intent,
        "putExtra",
        "(Ljava/lang/String;Ljava/lang/String;)Landroid/content/Intent;",
        &[JValue::Object(&extra_key), JValue::Object(&extra_value)],
    )?;

    match env.call_method(
        &activity,
        "startActivity",
        "(Landroid/content/Intent;)V",
        &[JValue::Object(&intent)],
    ) {
        Ok(_) => {}
        Err(_) => {
            env.exception_clear()?;
            dure_warn!("Failed to open developer settings");
        }
    }

    Ok(())
}

/// Open Google Play Store app for a specific package
#[cfg(target_os = "android")]
pub fn open_android_vending(package_name: &str) {
    if let Err(e) = open_android_vending_inner(package_name) {
        dure_error!("Failed to open Google Play Store: {}", e);
    }
}

#[cfg(target_os = "android")]
fn open_android_vending_inner(package_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let ctx = ndk_context::android_context();
    let vm = unsafe { jni::JavaVM::from_raw(ctx.vm() as _) }?;
    let activity = unsafe { jni::objects::JObject::from_raw(ctx.context() as _) };
    let mut env = vm.attach_current_thread()?;

    let intent_class = env.find_class("android/content/Intent")?;
    let uri_class = env.find_class("android/net/Uri")?;

    // Try to open the app details directly in the Google Play Store app
    let market_uri = format!("market://details?id={}", package_name);
    let market_uri_str = env.new_string(&market_uri)?;
    let market_uri_obj = env.call_static_method(
        &uri_class,
        "parse",
        "(Ljava/lang/String;)Landroid/net/Uri;",
        &[JValue::Object(&market_uri_str)],
    )?;
    let market_uri_parsed = market_uri_obj.l()?;

    let action_view = env.new_string("android.intent.action.VIEW")?;
    let intent = env.new_object(
        &intent_class,
        "(Ljava/lang/String;Landroid/net/Uri;)V",
        &[JValue::Object(&action_view), JValue::Object(&market_uri_parsed)],
    )?;

    // Set package to specifically target the Play Store app
    let vending_package = env.new_string("com.android.vending")?;
    env.call_method(
        &intent,
        "setPackage",
        "(Ljava/lang/String;)Landroid/content/Intent;",
        &[JValue::Object(&vending_package)],
    )?;

    match env.call_method(
        &activity,
        "startActivity",
        "(Landroid/content/Intent;)V",
        &[JValue::Object(&intent)],
    ) {
        Ok(_) => Ok(()),
        Err(_) => {
            // If the Play Store app is not installed, open in a web browser
            env.exception_clear()?;
            let web_url = env.new_string("https://play.google.com")?;
            let web_uri = env.call_static_method(
                &uri_class,
                "parse",
                "(Ljava/lang/String;)Landroid/net/Uri;",
                &[JValue::Object(&web_url)],
            )?;
            let web_uri_obj = web_uri.l()?;

            let web_action_view = env.new_string("android.intent.action.VIEW")?;
            let web_intent = env.new_object(
                &intent_class,
                "(Ljava/lang/String;Landroid/net/Uri;)V",
                &[JValue::Object(&web_action_view), JValue::Object(&web_uri_obj)],
            )?;

            env.call_method(
                &activity,
                "startActivity",
                "(Landroid/content/Intent;)V",
                &[JValue::Object(&web_intent)],
            )?;
            Ok(())
        }
    }
}

/// Open dure setup documentation page
#[cfg(target_os = "android")]
pub fn open_setup_page() {
    if let Err(e) = open_setup_page_inner() {
        dure_error!("Failed to open setup page: {}", e);
    }
}

#[cfg(target_os = "android")]
fn open_setup_page_inner() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = ndk_context::android_context();
    let vm = unsafe { jni::JavaVM::from_raw(ctx.vm() as _) }?;
    let activity = unsafe { jni::objects::JObject::from_raw(ctx.context() as _) };
    let mut env = vm.attach_current_thread()?;

    let intent_class = env.find_class("android/content/Intent")?;

    // Open dure setup documentation page
    let uri_class = env.find_class("android/net/Uri")?;
    let url = env.new_string("https://dure.pages.dev/docs/setup")?;
    let uri = env.call_static_method(
        &uri_class,
        "parse",
        "(Ljava/lang/String;)Landroid/net/Uri;",
        &[JValue::Object(&url)],
    )?;
    let uri_obj = uri.l()?;

    let action_view = env.new_string("android.intent.action.VIEW")?;
    let view_intent = env.new_object(
        &intent_class,
        "(Ljava/lang/String;Landroid/net/Uri;)V",
        &[JValue::Object(&action_view), JValue::Object(&uri_obj)],
    )?;

    env.call_method(
        &activity,
        "startActivity",
        "(Landroid/content/Intent;)V",
        &[JValue::Object(&view_intent)],
    )?;

    Ok(())
}

// Non-Android stub implementations
#[cfg(not(target_os = "android"))]
pub fn open_build_number_settings() {
    dure_debug!("open_build_number_settings is only available on Android");
}

#[cfg(not(target_os = "android"))]
pub fn open_wireless_debugging_settings() {
    dure_debug!("open_wireless_debugging_settings is only available on Android");
}

#[cfg(not(target_os = "android"))]
pub fn open_android_vending(package_name: &str) {
    dure_debug!("open_android_vending({}) is only available on Android", package_name);
}

#[cfg(not(target_os = "android"))]
pub fn open_setup_page() {
    dure_debug!("open_setup_page is only available on Android");
}
