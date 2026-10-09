//! "Open when the computer starts", done the way each kind of build must:
//!
//! - direct download (Windows, macOS, Linux): the autostart plugin
//!   (registry Run key / Launch Agent / .desktop file), started with `--hidden`
//! - Mac App Store: the system login-item service, the only way a sandboxed
//!   app may do it (macOS 13 or later)
//! - Microsoft Store: Windows owns the switch (Settings > Apps > Startup);
//!   the app can only take the user there

use tauri::AppHandle;

/// How the Settings screen should present the option.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Mode {
    /// A normal on/off switch.
    Switch,
    /// A button that opens the Windows "Startup apps" page.
    WindowsSettings,
}

pub const MODE: Mode = if cfg!(feature = "msstore") {
    Mode::WindowsSettings
} else {
    Mode::Switch
};

/// Turn it on or off. `Err` carries a short code the interface can explain.
#[allow(unused_variables)]
pub fn set(app: &AppHandle, on: bool) -> Result<(), String> {
    #[cfg(not(any(feature = "mas", feature = "msstore")))]
    {
        use tauri_plugin_autostart::ManagerExt;
        let launcher = app.autolaunch();
        let result = if on {
            launcher.enable()
        } else {
            launcher.disable()
        };
        return result.map_err(|_| "autostart".to_string());
    }
    #[cfg(all(feature = "mas", target_os = "macos"))]
    {
        use smappservice_rs::{AppService, ServiceType};
        let service = AppService::new(ServiceType::MainApp);
        let result = if on {
            service.register()
        } else {
            service.unregister()
        };
        return result.map_err(|_| "autostart".to_string());
    }
    #[allow(unreachable_code)]
    Err("autostart".into())
}

/// What the system says right now, if it can be asked.
#[allow(unused_variables)]
pub fn is_on(app: &AppHandle) -> Option<bool> {
    #[cfg(not(any(feature = "mas", feature = "msstore")))]
    {
        use tauri_plugin_autostart::ManagerExt;
        return app.autolaunch().is_enabled().ok();
    }
    #[cfg(all(feature = "mas", target_os = "macos"))]
    {
        use smappservice_rs::{AppService, ServiceStatus, ServiceType};
        return Some(AppService::new(ServiceType::MainApp).status() == ServiceStatus::Enabled);
    }
    #[allow(unreachable_code)]
    None
}

/// Microsoft Store build: open Settings > Apps > Startup.
pub fn open_system_settings(app: &AppHandle) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;
    if cfg!(windows) {
        app.opener()
            .open_url("ms-settings:startupapps", None::<&str>)
            .map_err(|e| e.to_string())
    } else {
        Err("autostart".into())
    }
}
