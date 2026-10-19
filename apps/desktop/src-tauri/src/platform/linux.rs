//! GNOME Wayland focused-application and idle adapter.
//! This module has no global keyboard hook, root access, or X11 fallback.
//! Requires user-installed, enabled Workforce GNOME Shell extension.
use super::{ActiveWindowInfo, CapabilityRow, Permission, PermissionState};
use crate::settings::Settings;
use zbus::blocking::{Connection, Proxy};

const FOCUS_NAME: &str = "org.workforceintelligence.Focus";
const FOCUS_PATH: &str = "/org/workforceintelligence/Focus";
const FOCUS_INTERFACE: &str = "org.workforceintelligence.Focus1";

pub fn active_window() -> Option<ActiveWindowInfo> {
    let conn = Connection::session().ok()?;
    let proxy = Proxy::new(&conn, FOCUS_NAME, FOCUS_PATH, FOCUS_INTERFACE).ok()?;
    let (app_id, _at_ms): (String, i64) = proxy.call("GetFocus", &()).ok()?;
    let app_id = app_id.trim();
    if app_id.is_empty() {
        return None;
    }
    Some(ActiveWindowInfo { app_name: app_id.to_owned(), title: None, pid: 0 })
}

/// Mutter reports milliseconds since last session input. Errors mean unknown,
/// never a false claim that the employee was active.
pub fn idle_seconds() -> f64 {
    (|| {
        let conn = Connection::session().ok()?;
        let proxy = Proxy::new(
            &conn,
            "org.gnome.Mutter.IdleMonitor",
            "/org/gnome/Mutter/IdleMonitor/Core",
            "org.gnome.Mutter.IdleMonitor",
        ).ok()?;
        let ms: u64 = proxy.call("GetIdletime", &()).ok()?;
        Some(ms as f64 / 1000.0)
    })().unwrap_or(f64::INFINITY)
}

pub fn capability_rows(_s: &Settings) -> Vec<CapabilityRow> {
    let focus_available = active_window().is_some();
    vec![
        CapabilityRow {
            key: "gnome_focus".into(),
            label: "GNOME application focus".into(),
            description: "Requires an explicitly enabled, user-session GNOME extension.".into(),
            state: if focus_available { PermissionState::Granted } else { PermissionState::Denied },
            required: true, can_request: false, can_open_settings: false,
        },
        CapabilityRow {
            key: "screencast".into(),
            label: "GNOME ScreenCast portal".into(),
            description: "Screenshot collection disabled until authorized portal capture is integrated.".into(),
            state: PermissionState::Denied,
            required: false, can_request: false, can_open_settings: false,
        },
        CapabilityRow {
            key: "input_counts".into(),
            label: "Approved keyboard and mouse counts".into(),
            description: "Requires a separately enrolled, authorized system input counter.".into(),
            state: PermissionState::Denied,
            required: false, can_request: false, can_open_settings: false,
        },
    ]
}

pub fn permission_status(_p: Permission) -> PermissionState { PermissionState::Denied }
pub fn open_settings(_p: Permission) {}
pub fn request_screen_recording() -> bool { false }
pub fn request_input_monitoring() -> bool { false }
pub fn request_accessibility() -> bool { false }
/// Stub needed by the shared tracker source; Linux does not install a keyboard hook.
pub fn run_keyboard_tap() -> bool { false }
