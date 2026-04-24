use std::path::PathBuf;
use std::process::Command;
use tauri::Manager;
use tauri_plugin_dialog::DialogExt;

use crate::config::{profile_to_cfg, Profile};

// ── Launch Hatari ─────────────────────────────────────────────────────────────

#[tauri::command]
pub fn launch_hatari(
    hatari_path: String,
    profile: Profile,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let cfg_content = profile_to_cfg(&profile);

    // Write to both config locations Hatari checks on macOS
    let home = std::env::var("HOME")
        .map_err(|_| "Cannot determine home directory".to_string())?;

    let cfg_locations = vec![
        std::path::PathBuf::from(&home).join(".config").join("hatari").join("hatari.cfg"),
        std::path::PathBuf::from(&home).join("Library").join("Application Support").join("Hatari").join("hatari.cfg"),
    ];

    for cfg_path in &cfg_locations {
        if let Some(parent) = cfg_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config dir: {e}"))?;
        }
        std::fs::write(cfg_path, &cfg_content)
            .map_err(|e| format!("Failed to write Hatari config to {:?}: {e}", cfg_path))?;
    }

    let hatari = if hatari_path.is_empty() {
        detect_hatari_path()
    } else {
        Some(PathBuf::from(&hatari_path))
    };

    let hatari = hatari.ok_or_else(|| {
        "Hatari not found. Please set the Hatari path in Settings.".to_string()
    })?;

    // Spawn Hatari detached from our process, with a clean environment
    // so it doesn't inherit anything from the Tauri dev server context
    let mut cmd = Command::new(&hatari);
    cmd.current_dir(&home);

    // On macOS, clear DYLD and related env vars that can leak from dev server
    cmd.env_remove("DYLD_INSERT_LIBRARIES");
    cmd.env_remove("DYLD_LIBRARY_PATH");
    cmd.env_remove("DYLD_FRAMEWORK_PATH");

    cmd.spawn()
        .map_err(|e| format!("Failed to launch Hatari: {e}"))?;

    Ok(())
}

// ── Detect Hatari installation ────────────────────────────────────────────────

#[tauri::command]
pub fn detect_hatari() -> Option<String> {
    detect_hatari_path().map(|p| p.to_string_lossy().to_string())
}

fn detect_hatari_path() -> Option<PathBuf> {
    // Common install locations on macOS and Windows
    let candidates: &[&str] = &[
        // macOS — Homebrew and app bundle
        "/usr/local/bin/hatari",
        "/opt/homebrew/bin/hatari",
        "/Applications/Hatari.app/Contents/MacOS/hatari",
        // Windows
        r"C:\Program Files\Hatari\hatari.exe",
        r"C:\Program Files (x86)\Hatari\hatari.exe",
    ];

    for path in candidates {
        let p = PathBuf::from(path);
        if p.exists() {
            return Some(p);
        }
    }

    // Fall back to PATH lookup
    which_hatari()
}

fn which_hatari() -> Option<PathBuf> {
    let output = Command::new("which")
        .arg("hatari")
        .output()
        .ok()?;

    if output.status.success() {
        let path = String::from_utf8(output.stdout).ok()?;
        let path = path.trim();
        if !path.is_empty() {
            return Some(PathBuf::from(path));
        }
    }
    None
}

// ── Profile persistence ───────────────────────────────────────────────────────

#[tauri::command]
pub fn save_profiles(
    profiles: Vec<Profile>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let data_dir = app.path().app_data_dir()
        .map_err(|e| format!("Cannot find app data dir: {e}"))?;

    std::fs::create_dir_all(&data_dir)
        .map_err(|e| format!("Failed to create data dir: {e}"))?;

    let path = data_dir.join("profiles.json");
    let json = serde_json::to_string_pretty(&profiles)
        .map_err(|e| format!("Failed to serialise profiles: {e}"))?;

    std::fs::write(&path, json)
        .map_err(|e| format!("Failed to write profiles: {e}"))?;

    Ok(())
}

#[tauri::command]
pub fn load_profiles(app: tauri::AppHandle) -> Result<Vec<Profile>, String> {
    let data_dir = app.path().app_data_dir()
        .map_err(|e| format!("Cannot find app data dir: {e}"))?;

    let path = data_dir.join("profiles.json");

    if !path.exists() {
        return Ok(vec![]);   // first run — no saved profiles yet
    }

    let json = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read profiles: {e}"))?;

    let profiles: Vec<Profile> = serde_json::from_str(&json)
        .map_err(|e| format!("Failed to parse profiles: {e}"))?;

    Ok(profiles)
}

// ── ROM folder scanning ───────────────────────────────────────────────────────

#[derive(Debug, Clone, serde::Serialize)]
pub struct DetectedRom {
    pub path:     String,
    pub filename: String,
    #[serde(rename = "tosVersion")]
    pub tos_version: Option<String>,
    pub machine:  Option<String>,
    pub size:     u64,
}

#[tauri::command]
pub fn scan_rom_folder(folder: String) -> Result<Vec<DetectedRom>, String> {
    if folder.is_empty() {
        return Ok(vec![]);
    }

    let path = std::path::Path::new(&folder);
    if !path.is_dir() {
        return Err(format!("Not a directory: {}", folder));
    }

    let mut detected = Vec::new();

    // Walk folder looking for ROM-ish files
    let entries = std::fs::read_dir(path)
        .map_err(|e| format!("Cannot read folder: {e}"))?;

    for entry in entries.flatten() {
        let entry_path = entry.path();
        if !entry_path.is_file() {
            continue;
        }

        let filename = entry_path.file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.to_string())
            .unwrap_or_default();

        let ext_lower = entry_path.extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_lowercase())
            .unwrap_or_default();

        // Only consider likely ROM extensions
        if !matches!(ext_lower.as_str(), "img" | "rom" | "bin") {
            continue;
        }

        // File size check — real TOS ROMs are 192KB, 256KB, 512KB or 1024KB
        let metadata = std::fs::metadata(&entry_path)
            .map_err(|e| format!("Cannot stat {filename}: {e}"))?;
        let size = metadata.len();

        if !matches!(size, 196608 | 262144 | 524288 | 1048576) {
            continue;
        }

        let (tos_version, machine) = detect_tos_from_filename(&filename);

        detected.push(DetectedRom {
            path:        entry_path.to_string_lossy().to_string(),
            filename,
            tos_version,
            machine,
            size,
        });
    }

    // Sort by filename
    detected.sort_by(|a, b| a.filename.cmp(&b.filename));

    Ok(detected)
}

/// Best-effort filename-based TOS detection.
/// Returns (tos_version, machine_type) where possible.
fn detect_tos_from_filename(filename: &str) -> (Option<String>, Option<String>) {
    let lower = filename.to_lowercase();

    // EmuTOS
    if lower.contains("emutos") || lower.starts_with("etos") {
        return (Some("EmuTOS".to_string()), None);
    }

    // Standard TOS version detection — match "tos" + version number
    let ver_patterns: &[(&str, &str, &str)] = &[
        // (pattern to match, TOS version, machine)
        ("tos 1.00",  "TOS 1.00", "ST"),
        ("tos100",    "TOS 1.00", "ST"),
        ("tos 1.02",  "TOS 1.02", "ST"),
        ("tos102",    "TOS 1.02", "ST"),
        ("tos 1.04",  "TOS 1.04", "ST"),
        ("tos104",    "TOS 1.04", "ST"),
        ("tos 1.06",  "TOS 1.06", "STE"),
        ("tos106",    "TOS 1.06", "STE"),
        ("tos 1.62",  "TOS 1.62", "STE"),
        ("tos162",    "TOS 1.62", "STE"),
        ("tos 2.05",  "TOS 2.05", "MegaSTE"),
        ("tos205",    "TOS 2.05", "MegaSTE"),
        ("tos 2.06",  "TOS 2.06", "MegaSTE"),
        ("tos206",    "TOS 2.06", "MegaSTE"),
        ("tos 3.06",  "TOS 3.06", "TT"),
        ("tos306",    "TOS 3.06", "TT"),
        ("tos 4.04",  "TOS 4.04", "Falcon"),
        ("tos404",    "TOS 4.04", "Falcon"),
        ("tos 4.02",  "TOS 4.02", "Falcon"),
        ("tos402",    "TOS 4.02", "Falcon"),
    ];

    for (pattern, ver, machine) in ver_patterns {
        if lower.contains(pattern) {
            return (Some(ver.to_string()), Some(machine.to_string()));
        }
    }

    // No match
    (None, None)
}

// ── Settings persistence ──────────────────────────────────────────────────────

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AppSettings {
    #[serde(rename = "hatariPath")]
    pub hatari_path:   String,
    #[serde(rename = "romsFolder")]
    pub roms_folder:   String,
    #[serde(rename = "floppyFolder")]
    pub floppy_folder: String,
    pub theme:         String,
}

#[tauri::command]
pub fn save_settings(
    settings: AppSettings,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let data_dir = app.path().app_data_dir()
        .map_err(|e| format!("Cannot find app data dir: {e}"))?;

    std::fs::create_dir_all(&data_dir)
        .map_err(|e| format!("Failed to create data dir: {e}"))?;

    let path = data_dir.join("settings.json");
    let json = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("Failed to serialise settings: {e}"))?;

    std::fs::write(&path, json)
        .map_err(|e| format!("Failed to write settings: {e}"))?;

    Ok(())
}

#[tauri::command]
pub fn load_settings(app: tauri::AppHandle) -> Result<Option<AppSettings>, String> {
    let data_dir = app.path().app_data_dir()
        .map_err(|e| format!("Cannot find app data dir: {e}"))?;

    let path = data_dir.join("settings.json");

    if !path.exists() {
        return Ok(None);
    }

    let json = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read settings: {e}"))?;

    let settings: AppSettings = serde_json::from_str(&json)
        .map_err(|e| format!("Failed to parse settings: {e}"))?;

    Ok(Some(settings))
}

// ── File / folder pickers ─────────────────────────────────────────────────────
// These are now handled directly by @tauri-apps/plugin-dialog on the frontend.
// Rust-side pick_file / pick_folder commands are no longer needed.