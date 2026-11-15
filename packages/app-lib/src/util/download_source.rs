//! Chooses where Minecraft files are downloaded from.
//!
//! Mojang is the default source. When BMCLAPI is selected, the Mojang URLs that
//! the version manifest and libraries point at are rewritten to their BMCLAPI
//! mirrors, which are much faster from mainland China.

use std::borrow::Cow;
use std::sync::atomic::{AtomicBool, Ordering};

/// Key used to store the choice in the app metadata table.
pub const SETTING_KEY: &str = "version_download_source";
pub const SOURCE_MOJANG: &str = "mojang";
pub const SOURCE_BMCLAPI: &str = "bmclapi";

const BMCLAPI: &str = "https://bmclapi2.bangbang93.com";
const MOJANG_MANIFEST_HOSTS: [&str; 2] = [
    "https://launchermeta.mojang.com",
    "https://piston-meta.mojang.com",
];
const MOJANG_ASSETS: &str = "https://resources.download.minecraft.net";
const MOJANG_LIBRARIES: &str = "https://libraries.minecraft.net";
const MOJANG_OBJECTS: &str = "https://piston-data.mojang.com";
const MOJANG_VERSION_INFO_HOSTS: [&str; 2] = [
    "https://piston-meta.mojang.com",
    "https://launchermeta.mojang.com",
];

static USE_BMCLAPI: AtomicBool = AtomicBool::new(false);

pub fn is_bmclapi() -> bool {
    USE_BMCLAPI.load(Ordering::Relaxed)
}

pub fn set_bmclapi(enabled: bool) {
    USE_BMCLAPI.store(enabled, Ordering::Relaxed);
}

/// The source name stored in settings.
pub fn source() -> &'static str {
    if is_bmclapi() {
        SOURCE_BMCLAPI
    } else {
        SOURCE_MOJANG
    }
}

pub async fn get_source() -> crate::Result<String> {
    let state = crate::State::get().await?;
    let stored =
        crate::state::content_store::setting(&state.pool, SETTING_KEY).await?;
    Ok(stored
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| SOURCE_MOJANG.to_string()))
}

pub async fn set_source(source: &str) -> crate::Result<()> {
    let normalized = if source == SOURCE_BMCLAPI {
        SOURCE_BMCLAPI
    } else {
        SOURCE_MOJANG
    };
    let state = crate::State::get().await?;
    crate::state::content_store::set_setting(
        &state.pool,
        SETTING_KEY,
        normalized,
    )
    .await?;
    set_bmclapi(normalized == SOURCE_BMCLAPI);
    tracing::info!(source = normalized, "Minecraft download source changed");
    Ok(())
}

/// Applies the stored source setting to the process-wide switch.
pub async fn load_source(pool: &sqlx::SqlitePool) -> crate::Result<()> {
    let stored =
        crate::state::content_store::setting(pool, SETTING_KEY).await?;
    set_bmclapi(stored.as_deref() == Some(SOURCE_BMCLAPI));
    Ok(())
}

/// Mirrors the URL of a version manifest entry's version JSON.
pub fn version_info_url<'a>(url: &'a str, id: &str) -> Cow<'a, str> {
    if is_bmclapi()
        && MOJANG_VERSION_INFO_HOSTS
            .iter()
            .any(|host| url.starts_with(host))
    {
        return Cow::Owned(format!("{BMCLAPI}/version/{id}/json"));
    }
    Cow::Borrowed(url)
}

/// Rewrites a Mojang download URL to its BMCLAPI mirror.
///
/// `version` is the Minecraft version the URL belongs to, when known, so that
/// version specific files (the client jar) can be mirrored as well.
pub fn rewrite<'a>(url: &'a str, version: Option<&str>) -> Cow<'a, str> {
    if !is_bmclapi() {
        return Cow::Borrowed(url);
    }

    if MOJANG_MANIFEST_HOSTS
        .iter()
        .any(|host| url.starts_with(host) && url.ends_with("/version_manifest_v2.json"))
    {
        return Cow::Owned(format!("{BMCLAPI}/mc/game/version_manifest_v2.json"));
    }

    if let Some(relative) = url.strip_prefix(MOJANG_ASSETS) {
        return Cow::Owned(format!("{BMCLAPI}/assets{relative}"));
    }

    if let Some(relative) = url.strip_prefix(MOJANG_LIBRARIES) {
        return Cow::Owned(format!("{BMCLAPI}/maven{relative}"));
    }

    if let Some(version) = version
        && url.starts_with(MOJANG_OBJECTS)
        && url.ends_with("/client.jar")
    {
        return Cow::Owned(format!("{BMCLAPI}/version/{version}/client"));
    }

    Cow::Borrowed(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn leaves_urls_alone_for_mojang() {
        set_bmclapi(false);
        assert_eq!(
            rewrite(
                "https://libraries.minecraft.net/com/example/a.jar",
                None
            ),
            "https://libraries.minecraft.net/com/example/a.jar"
        );
    }

    #[test]
    fn mirrors_minecraft_urls() {
        set_bmclapi(true);
        assert_eq!(
            rewrite(
                "https://launchermeta.mojang.com/mc/game/version_manifest_v2.json",
                None
            ),
            "https://bmclapi2.bangbang93.com/mc/game/version_manifest_v2.json"
        );
        assert_eq!(
            rewrite(
                "https://resources.download.minecraft.net/ab/abcdef",
                None
            ),
            "https://bmclapi2.bangbang93.com/assets/ab/abcdef"
        );
        assert_eq!(
            rewrite("https://libraries.minecraft.net/com/example/a.jar", None),
            "https://bmclapi2.bangbang93.com/maven/com/example/a.jar"
        );
        assert_eq!(
            rewrite(
                "https://piston-data.mojang.com/v1/objects/abc/client.jar",
                Some("1.21.4")
            ),
            "https://bmclapi2.bangbang93.com/version/1.21.4/client"
        );
        assert_eq!(
            version_info_url(
                "https://piston-meta.mojang.com/v1/packages/abcdef/1.21.4.json",
                "1.21.4"
            ),
            "https://bmclapi2.bangbang93.com/version/1.21.4/json"
        );
        set_bmclapi(false);
    }
}
