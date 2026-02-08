// Runtime hot-reload configuration system.
// Watches a config file via polling and updates global atomic values.
// Zero external dependencies — uses only std.

use log::{info, warn};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::{Duration, SystemTime};

// ─── Global Atomic Config Values ───────────────────────────────────────────

static MIN_BOTS: AtomicUsize = AtomicUsize::new(0);
static MAX_BOTS: AtomicUsize = AtomicUsize::new(0);
static BOT_PERCENT: AtomicUsize = AtomicUsize::new(0);
static BOT_ALLIANCE: AtomicBool = AtomicBool::new(false);
static FACTION_MODE: AtomicBool = AtomicBool::new(false);
static CONFIG_LOADED: AtomicBool = AtomicBool::new(false);

// ─── Public Accessors ──────────────────────────────────────────────────────

pub fn hot_min_bots() -> Option<usize> {
    if !CONFIG_LOADED.load(Ordering::Relaxed) { return None; }
    let v = MIN_BOTS.load(Ordering::Relaxed);
    if v > 0 { Some(v) } else { None }
}

pub fn hot_max_bots() -> Option<usize> {
    if !CONFIG_LOADED.load(Ordering::Relaxed) { return None; }
    let v = MAX_BOTS.load(Ordering::Relaxed);
    if v > 0 { Some(v) } else { None }
}

pub fn hot_bot_percent() -> Option<usize> {
    if !CONFIG_LOADED.load(Ordering::Relaxed) { return None; }
    let v = BOT_PERCENT.load(Ordering::Relaxed);
    if v > 0 { Some(v) } else { None }
}

pub fn hot_bot_alliance_enabled() -> bool {
    BOT_ALLIANCE.load(Ordering::Relaxed)
}

pub fn hot_faction_mode() -> bool {
    FACTION_MODE.load(Ordering::Relaxed)
}

/// Force-enable faction mode (e.g. from CLI `--faction-mode`).
pub fn set_faction_mode(enabled: bool) {
    FACTION_MODE.store(enabled, Ordering::Relaxed);
}


// ─── Config Parsing (simple key=value, ignoring [sections]) ────────────────

fn load_and_apply(path: &Path) {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            warn!("[HotReload] Failed to read {:?}: {}", path, e);
            return;
        }
    };

    let mut min_bots: Option<usize> = None;
    let mut max_bots: Option<usize> = None;
    let mut bot_percent: Option<usize> = None;
    let mut alliance = false;
    let mut faction_mode = false;

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with('[') {
            continue;
        }
        if let Some((key, val)) = line.split_once('=') {
            let key = key.trim();
            let val = val.trim();
            match key {
                "min_bots" => min_bots = val.parse().ok(),
                "max_bots" => max_bots = val.parse().ok(),
                "bot_percent" => bot_percent = val.parse().ok(),
                "alliance_enabled" => alliance = val == "true" || val == "1",
                "faction_mode" => faction_mode = val == "true" || val == "1",
                _ => {}
            }
        }
    }

    if let Some(v) = min_bots { MIN_BOTS.store(v, Ordering::Relaxed); }
    if let Some(v) = max_bots { MAX_BOTS.store(v, Ordering::Relaxed); }
    if let Some(v) = bot_percent { BOT_PERCENT.store(v, Ordering::Relaxed); }
    BOT_ALLIANCE.store(alliance, Ordering::Relaxed);
    FACTION_MODE.store(faction_mode, Ordering::Relaxed);
    CONFIG_LOADED.store(true, Ordering::Relaxed);

    info!(
        "[HotReload] Config applied: min_bots={:?}, max_bots={:?}, bot_percent={:?}, alliance={}, faction_mode={}",
        min_bots, max_bots, bot_percent, alliance, faction_mode,
    );
}

// ─── File Watcher (polling) ────────────────────────────────────────────────

/// Starts the config file watcher. Call from main() before the game server entry point.
/// If the config file doesn't exist, a default one is created.
/// A background thread polls for file changes every 2 seconds.
pub fn start_config_watcher(config_path: Option<String>) {
    let path = config_path
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("config.toml"));

    // Create default config if it doesn't exist.
    if !path.exists() {
        let default_config = r#"# MK48 Plus Runtime Configuration
# Changes to this file are applied automatically without restarting the server.
# The file is polled every 2 seconds for changes.

# Bot settings
min_bots = 250
# max_bots = 400
# bot_percent = 50
alliance_enabled = false
faction_mode = false

# Minimap settings
# minimap_speed_coeff = 30.0
# minimap_default_mult = 2.0
# minimap_min_range = 500.0
# minimap_max_range = 5000.0
"#;
        match std::fs::write(&path, default_config) {
            Ok(_) => info!("[HotReload] Created default config file at {:?}", path),
            Err(e) => {
                warn!("[HotReload] Could not create default config file: {}", e);
                return;
            }
        }
    }

    // Load initial config.
    load_and_apply(&path);

    // Spawn polling thread.
    std::thread::spawn(move || {
        let mut last_modified: Option<SystemTime> = std::fs::metadata(&path)
            .and_then(|m| m.modified())
            .ok();

        loop {
            std::thread::sleep(Duration::from_secs(2));

            let current_modified = match std::fs::metadata(&path).and_then(|m| m.modified()) {
                Ok(t) => t,
                Err(_) => continue,
            };

            let changed = match last_modified {
                Some(prev) => current_modified != prev,
                None => true,
            };

            if changed {
                info!("[HotReload] Config file changed, reloading...");
                load_and_apply(&path);
                last_modified = Some(current_modified);
            }
        }
    });
}
