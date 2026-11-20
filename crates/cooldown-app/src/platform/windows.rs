//! Windows: the target platform.

use std::path::PathBuf;

pub const NAME: &str = "windows";

/// Windows: a left click on the notification area icon opens the window; the menu is on the right click.
pub const TRAY_MENU_ON_LEFT_CLICK: bool = false;

/// `%LOCALAPPDATA%\cooldown.lol\logs`
pub fn log_dir() -> PathBuf {
    std::env::var_os("LOCALAPPDATA")
        .map_or_else(|| PathBuf::from("logs"), PathBuf::from)
        .join("cooldown.lol")
        .join("logs")
}

/// `%APPDATA%\cooldown.lol`
pub fn data_dir() -> PathBuf {
    std::env::var_os("APPDATA")
        .map_or_else(|| PathBuf::from("data"), PathBuf::from)
        .join("cooldown.lol")
}

/// Windows-specific window setup.
///
/// TODO(windows): overlay behaviour (always on top, click-through, positioning
/// over the game window). See docs/Roadmap.md.
pub fn configure_main_window(_window: &slint::Window) {
    tracing::debug!("no Windows-specific window configuration yet");
}

/// Whether to play screen transitions and the headline typing: off when the player
/// turned off Windows' "Animation effects" (Settings → Accessibility → Visual
/// effects), which writes `MinAnimate = 0` in `HKCU\Control Panel\Desktop\WindowMetrics`.
/// Read once at launch; on when it cannot be read.
pub fn animations_enabled() -> bool {
    use winreg::RegKey;
    use winreg::enums::HKEY_CURRENT_USER;

    let enabled = RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey("Control Panel\\Desktop\\WindowMetrics")
        .and_then(|key| key.get_value::<String, _>("MinAnimate"))
        .map_or(true, |value| value.trim() != "0");
    tracing::info!(enabled, "Windows animation effects");
    enabled
}
