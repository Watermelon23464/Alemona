use crate::api::Result;
use theseus::metadata::{
    LiteLoaderBuild, Manifest, OptifineVersion, VersionManifest,
};

pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("metadata")
        .invoke_handler(tauri::generate_handler![
            metadata_get_game_versions,
            metadata_get_loader_versions,
            metadata_get_optifine_versions,
            metadata_get_liteloader_build,
        ])
        .build()
}

/// Gets the game versions from launcher metadata
#[tauri::command]
pub async fn metadata_get_game_versions() -> Result<VersionManifest> {
    Ok(theseus::metadata::get_minecraft_versions().await?)
}

/// Gets the fabric versions from launcher metadata
#[tauri::command]
pub async fn metadata_get_loader_versions(loader: &str) -> Result<Manifest> {
    Ok(theseus::metadata::get_loader_versions(loader).await?)
}

/// Gets the OptiFine builds for a Minecraft version
#[tauri::command]
pub async fn metadata_get_optifine_versions(
    mc_version: &str,
) -> Result<Vec<OptifineVersion>> {
    Ok(theseus::metadata::get_optifine_versions(mc_version).await?)
}

/// Gets the LiteLoader build for a Minecraft version, if one exists
#[tauri::command]
pub async fn metadata_get_liteloader_build(
    mc_version: &str,
) -> Result<Option<LiteLoaderBuild>> {
    Ok(theseus::metadata::get_liteloader_build(mc_version).await?)
}
