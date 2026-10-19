//! GNOME Wayland screenshot adapter: only reads user-authorized portal frames.
//! Requires separately installed, reviewed `wayland-screenshot` CLI. No X11 fallback.
//! Upstream CLI: https://github.com/dtg01100/wayland-screenshot
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{atomic::Ordering, Arc};
use std::thread;
use std::time::Duration;
use crate::settings::SettingsState;
use crate::storage::{Db, Screenshot};
use crate::trackers::{self, TrackerControl};

fn sensitive_focus(control: &TrackerControl) -> bool {
    let Some(w) = crate::platform::active_window() else { return true; };
    let name = w.app_name.to_lowercase();
    let skips = control.screenshot_skip_apps.read().unwrap();
    skips.iter().any(|p| !p.trim().is_empty() && name.contains(&p.to_lowercase()))
}
pub fn portal_stream_ready() -> bool {
    Command::new("wayland-screenshot")
        .args(["cast", "status", "--json"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| serde_json::from_slice::<serde_json::Value>(&output.stdout).ok())
        .and_then(|v| v.get("running").and_then(|x| x.as_bool()))
        .unwrap_or(false)
}
pub fn capture_once(db: &Db, dir: &Path, control: &TrackerControl) -> usize {
    if control.paused.load(Ordering::Relaxed)
        || !control.capture_screenshots.load(Ordering::Relaxed)
        || sensitive_focus(control) || !portal_stream_ready() { return 0; }
    if std::fs::create_dir_all(dir).is_err() { return 0; }
    let now = crate::now_unix();
    let id = uuid::Uuid::new_v4();
    let png_path = dir.join(format!(".{now}-{id}.png"));
    let image_path = dir.join(format!("{now}-{id}.webp"));
    let result = (|| -> Result<(), String> {
        let output = Command::new("wayland-screenshot")
            .arg("snap").arg("-o").arg(&png_path)
            .output().map_err(|e| format!("portal capture: {e}"))?;
        if !output.status.success() { return Err("portal capture denied or unavailable".into()); }
        let png = std::fs::read(&png_path).map_err(|e| e.to_string())?;
        if png.len() > 40 * 1024 * 1024 { return Err("PNG exceeds input size limit".into()); }
        let img = xcap::image::load_from_memory(&png).map_err(|e| e.to_string())?.to_rgba8();
        if u64::from(img.width()) * u64::from(img.height()) > 20_000_000 {
            return Err("image exceeds pixel limit".into());
        }
        let (encoded, width, height) = trackers::compress_to_webp(&img);
        if encoded.len() > 200 * 1024 { return Err("WebP exceeds backend upload limit".into()); }
        std::fs::write(&image_path, encoded).map_err(|e| e.to_string())?;
        db.insert_screenshot(&Screenshot {
            ts: now, file_path: image_path.to_string_lossy().into_owned(),
            display_id: None, width: Some(width as i64), height: Some(height as i64),
        }).map_err(|e| e.to_string())?;
        Ok(())
    })();
    let _ = std::fs::remove_file(&png_path);
    if let Err(e) = result {
        let _ = std::fs::remove_file(&image_path);
        crate::log_warn!("linux_screenshot", "capture skipped: {e}");
        return 0;
    }
    1
}
pub fn start(db: Arc<Db>, control: Arc<TrackerControl>, settings: Arc<SettingsState>, dir: PathBuf) {
    thread::spawn(move || loop {
        let seconds = control.screenshot_interval_s.load(Ordering::Relaxed).max(30);
        let consented = settings.current.lock().unwrap().consented;
        let idle = crate::platform::idle_seconds();
        if consented && !control.paused.load(Ordering::Relaxed)
            && idle.is_finite()
            && idle < control.idle_threshold_s.load(Ordering::Relaxed) as f64 {
            let _ = capture_once(&db, &dir, &control);
        }
        thread::sleep(Duration::from_secs(seconds));
    });
}
