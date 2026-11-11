//! Where OpenToyStory2 keeps its own files. Everything the player owns lives in one per-user
//! folder, apart from the install (which an upgrade or uninstall replaces) and the game:
//!
//! ```text
//! %APPDATA%\OpenToyStory2\            (Linux: $XDG_CONFIG_HOME/opents2, macOS: ~/Library/Application Support/OpenToyStory2)
//!   opents2.toml                settings written by the launcher
//!   mods\<id>\mod.toml          one folder per mod, with its manifest (the example mods
//!                               are copied here once from `mods\` beside the launcher)
//!   disc\audio, disc\rtlibs     music and movies copied from a disc image
//!   last-run.log                the engine's output from the last launch
//! ```
//!
//! `OPENTS2_HOME` overrides the folder (tests, portable installs). Saves stay where the
//! engine keeps them (`TS2_SAVE_DIR`, else `%APPDATA%\ts2-rust`).
use std::path::PathBuf;

/// The per-user OpenToyStory2 folder. One left by an earlier version under the old name
/// (OpenTS2) is moved here the first time.
pub fn home() -> PathBuf {
    if let Some(p) = std::env::var_os("OPENTS2_HOME") {
        return p.into();
    }
    let home = default_home();
    if let Some(old) = old_home().filter(|o| !home.exists() && o.is_dir()) {
        let _ = std::fs::rename(&old, &home);
    }
    home
}

/// A path the settings saved inside the old OpenTS2 folder (the disc copy), moved to where
/// that folder now is. None for any other path.
pub fn moved_from_old_home(p: &std::path::Path) -> Option<PathBuf> {
    let old = old_home()?;
    let rest = p.strip_prefix(&old).ok()?;
    Some(default_home().join(rest))
}

/// Where earlier versions, called OpenTS2, kept the folder (Windows and macOS; the Linux
/// folder was already `opents2`).
fn old_home() -> Option<PathBuf> {
    #[cfg(windows)]
    return std::env::var_os("APPDATA").map(|p| PathBuf::from(p).join("OpenTS2"));
    #[cfg(target_os = "macos")]
    return std::env::var_os("HOME").map(|h| PathBuf::from(h).join("Library/Application Support/OpenTS2"));
    #[allow(unreachable_code)]
    None
}

fn default_home() -> PathBuf {
    #[cfg(windows)]
    let base = std::env::var_os("APPDATA").map(PathBuf::from).map(|p| p.join("OpenToyStory2"));
    #[cfg(target_os = "macos")]
    let base = std::env::var_os("HOME").map(|h| PathBuf::from(h).join("Library/Application Support/OpenToyStory2"));
    #[cfg(all(unix, not(target_os = "macos")))]
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
        .map(|p| p.join("opents2"));
    base.unwrap_or_else(|| PathBuf::from("OpenToyStory2"))
}

pub fn config_file() -> PathBuf {
    home().join("opents2.toml")
}

pub fn mods_dir() -> PathBuf {
    home().join("mods")
}

/// Music and movies copied from a disc image (`audio\`, `rtlibs\`).
pub fn disc_copy_dir() -> PathBuf {
    home().join("disc")
}

pub fn log_file() -> PathBuf {
    home().join("last-run.log")
}

/// The example mods that come with OpenToyStory2: `mods\` beside the launcher (the installer puts
/// them there), else, when running from a source checkout (`target\release\`), the
/// checkout's `mods\` folder.
pub fn bundled_mods_dir() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let dir = exe.parent()?;
    if dir.join("mods").is_dir() {
        return Some(dir.join("mods"));
    }
    dir.ancestors().take(4).find(|d| d.join("Cargo.toml").is_file() && d.join("mods").is_dir()).map(|d| d.join("mods"))
}

/// The engine executable, installed beside the launcher.
pub fn engine_exe() -> PathBuf {
    let name = if cfg!(windows) { "ts2-viewer.exe" } else { "ts2-viewer" };
    std::env::current_exe()
        .ok()
        .and_then(|e| e.parent().map(|d| d.join(name)))
        .unwrap_or_else(|| PathBuf::from(name))
}
