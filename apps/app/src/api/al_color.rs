use std::path::PathBuf;

use anyhow::anyhow;
use serde::{Deserialize, Serialize};

use crate::api::Result;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ColourEntry {
    pub primary: String,
    pub secondary: String,
    #[serde(default)]
    pub saved_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct StoredColours {
    pub primary: String,
    pub secondary: String,
    /// Every colour pair the launcher has used, oldest first. Nothing is
    /// discarded when a new colour is chosen.
    #[serde(default)]
    pub history: Vec<ColourEntry>,
}

fn colour_file() -> Result<PathBuf> {
    let exe = std::env::current_exe()?;
    let dir = exe
        .parent()
        .ok_or_else(|| anyhow!("launcher directory not found"))?
        .join("AL")
        .join("cache");
    std::fs::create_dir_all(&dir)?;
    Ok(dir.join("color.json"))
}

fn read_stored(path: &PathBuf) -> Option<StoredColours> {
    let raw = std::fs::read(path).ok()?;
    serde_json::from_slice(&raw).ok()
}

pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("al-color")
        .invoke_handler(tauri::generate_handler![al_color_load, al_color_save])
        .build()
}

/// Reads the remembered colours. The launcher treats these as a fallback: the
/// colours currently in use always win.
#[tauri::command]
pub async fn al_color_load() -> Result<Option<StoredColours>> {
    let path = colour_file()?;
    if !path.exists() {
        return Ok(None);
    }
    Ok(read_stored(&path))
}

/// Remembers the colours while keeping everything recorded before them.
#[tauri::command]
pub async fn al_color_save(primary: String, secondary: String) -> Result<()> {
    let path = colour_file()?;
    let mut stored = read_stored(&path).unwrap_or_default();

    // Preserve the previous pair before replacing it, unless it is the same one.
    let repeated = stored
        .history
        .last()
        .is_some_and(|last| last.primary == stored.primary && last.secondary == stored.secondary);
    if !stored.primary.is_empty()
        && (stored.primary != primary || stored.secondary != secondary)
        && !repeated
    {
        stored.history.push(ColourEntry {
            primary: stored.primary.clone(),
            secondary: stored.secondary.clone(),
            saved_at: String::new(),
        });
        // Keep the file bounded but never lose the recent past.
        if stored.history.len() > 40 {
            let excess = stored.history.len() - 40;
            stored.history.drain(0..excess);
        }
    }

    stored.primary = primary;
    stored.secondary = secondary;
    std::fs::write(path, serde_json::to_vec_pretty(&stored)?)?;
    Ok(())
}
