// Small JSON settings file in the app's config folder. Favorites live here, on this PC only.
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub favorites: Vec<u32>,
    /// Off means Steam starts with -silent: straight to the tray, no window.
    pub show_steam_window: bool,
    pub offline_mode: bool,
    pub web_api_key: String,
    pub view: String,
    pub sort: String,
    pub scope: String,
    pub show_software: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            favorites: Vec::new(),
            show_steam_window: false,
            offline_mode: false,
            web_api_key: String::new(),
            view: "grid".into(),
            sort: "name".into(),
            scope: "account".into(),
            show_software: true,
        }
    }
}

/// The app was called Lobby before; its settings lived under this identifier.
const OLD_IDENTIFIER: &str = "dev.patrick.lobby";

fn file(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_config_dir().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join("settings.json");
    if !path.exists() {
        if let Some(old) = dir.parent().map(|p| p.join(OLD_IDENTIFIER).join("settings.json")) {
            let _ = std::fs::copy(old, &path);
        }
    }
    Ok(path)
}

pub fn load(app: &AppHandle) -> Settings {
    file(app)
        .ok()
        .and_then(|f| std::fs::read_to_string(f).ok())
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save(app: &AppHandle, settings: &Settings) -> Result<(), String> {
    let json = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    std::fs::write(file(app)?, json).map_err(|e| e.to_string())
}
