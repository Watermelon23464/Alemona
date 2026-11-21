use crate::api::Result;
use theseus::terracotta::TerracottaInstall;

pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("terracotta")
        .invoke_handler(tauri::generate_handler![
            terracotta_get_install,
            terracotta_install,
            terracotta_launch,
            terracotta_state,
            terracotta_host,
            terracotta_join,
            terracotta_reset,
            terracotta_stop,
        ])
        .build()
}

/// Reports whether the Terracotta release has been downloaded already.
#[tauri::command]
pub async fn terracotta_get_install() -> Result<TerracottaInstall> {
    Ok(theseus::terracotta::get_install().await?)
}

/// Downloads and unpacks the official Terracotta release.
#[tauri::command]
pub async fn terracotta_install() -> Result<TerracottaInstall> {
    Ok(theseus::terracotta::install().await?)
}

/// Starts Terracotta and returns the HTTP API port.
#[tauri::command]
pub async fn terracotta_launch() -> Result<u16> {
    Ok(theseus::terracotta::launch().await?)
}

/// Reads the current room state.
#[tauri::command]
pub async fn terracotta_state() -> Result<serde_json::Value> {
    Ok(theseus::terracotta::state().await?)
}

/// Creates a room (host mode).
#[tauri::command]
pub async fn terracotta_host(player: Option<String>) -> Result<()> {
    Ok(theseus::terracotta::host(player.as_deref()).await?)
}

/// Joins a room with an invitation code.
#[tauri::command]
pub async fn terracotta_join(
    room: String,
    player: Option<String>,
) -> Result<()> {
    Ok(theseus::terracotta::join(&room, player.as_deref()).await?)
}

/// Returns Terracotta to the idle state.
#[tauri::command]
pub async fn terracotta_reset() -> Result<()> {
    Ok(theseus::terracotta::reset().await?)
}

/// Shuts Terracotta down.
#[tauri::command]
pub async fn terracotta_stop() -> Result<()> {
    Ok(theseus::terracotta::stop().await?)
}
