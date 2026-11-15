pub use crate::launcher::profile::{
    LoaderManifestMetadata, ResolvedVersion, loader_manifest_metadata,
    loader_manifest_metadata_from_cache_key, merge_partial_version,
};
use crate::state::CachedEntry;
use crate::{ErrorKind, State};
pub use minecraft_meta_manifest::fabriclike::Manifest;
pub use minecraft_meta_manifest::minecraft::Catalog as VersionManifest;
use serde::{Deserialize, Serialize};

const OPTIFINE_API: &str = "https://bmclapi2.bangbang93.com/optifine";
const LITELOADER_API: &str = "https://bmclapi2.bangbang93.com/liteloader";

/// One OptiFine build published for a Minecraft version.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OptifineVersion {
    pub patch: String,
    /// Build channel, e.g. `HD_U`.
    #[serde(rename = "type")]
    pub kind: String,
    pub filename: String,
    /// The Forge version this build is meant to run with, e.g. `Forge 47.2.18`
    /// or `Forge N/A` when it only works on vanilla.
    #[serde(default)]
    pub forge: Option<String>,
}

/// LiteLoader has a single build per Minecraft version, described by this data.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LiteLoaderBuild {
    #[serde(default)]
    pub mcversion: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub build: Option<String>,
    #[serde(default)]
    pub file: Option<String>,
    #[serde(rename = "tweakClass", default)]
    pub tweak_class: Option<String>,
    #[serde(default)]
    pub libraries: Vec<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct LiteLoaderResponse {
    #[serde(default)]
    pub mcversion: Option<String>,
    #[serde(default)]
    pub build: Option<LiteLoaderBuild>,
}

#[tracing::instrument]
pub async fn get_minecraft_versions() -> crate::Result<VersionManifest> {
    let state = State::get().await?;
    let minecraft_versions = CachedEntry::get_minecraft_manifest(
        None,
        &state.pool,
        &state.api_semaphore,
    )
    .await?
    .ok_or(ErrorKind::NoValueFor("minecraft versions".to_string()))?;

    Ok(minecraft_versions)
}

// #[tracing::instrument]
pub async fn get_loader_versions(loader: &str) -> crate::Result<Manifest> {
    let state = State::get().await?;
    let cache_key = loader_manifest_metadata(loader).cache_key;
    let loaders = CachedEntry::get_loader_manifest(
        &cache_key,
        None,
        &state.pool,
        &state.api_semaphore,
    )
    .await?
    .ok_or(ErrorKind::NoValueFor(format!("{loader} loader versions")))?;

    Ok(loaders.manifest)
}

/// OptiFine builds available for a Minecraft version, newest first.
pub async fn get_optifine_versions(
    mc_version: &str,
) -> crate::Result<Vec<OptifineVersion>> {
    let state = State::get().await?;
    let url = format!("{OPTIFINE_API}/{mc_version}");
    let bytes = crate::util::fetch::fetch(
        &url,
        None,
        None,
        None,
        &state.api_semaphore,
        &state.pool,
    )
    .await?;

    let mut versions: Vec<OptifineVersion> =
        serde_json::from_slice(bytes.as_ref())?;
    versions.reverse();
    Ok(versions)
}

/// The LiteLoader build for a Minecraft version, when one exists.
pub async fn get_liteloader_build(
    mc_version: &str,
) -> crate::Result<Option<LiteLoaderBuild>> {
    let state = State::get().await?;
    let url = format!("{LITELOADER_API}/list?mcversion={mc_version}");
    let bytes = crate::util::fetch::fetch(
        &url,
        None,
        None,
        None,
        &state.api_semaphore,
        &state.pool,
    )
    .await;

    let bytes = match bytes {
        Ok(bytes) => bytes,
        Err(_) => return Ok(None),
    };

    let response: LiteLoaderResponse = match serde_json::from_slice(bytes.as_ref())
    {
        Ok(response) => response,
        Err(_) => return Ok(None),
    };
    Ok(response.build.map(|mut build| {
        if build.mcversion.is_none() {
            build.mcversion = response.mcversion;
        }
        build
    }))
}
