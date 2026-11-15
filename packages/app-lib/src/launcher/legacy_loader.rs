//! Legacy tweaker loaders (LiteLoader, standalone OptiFine).
//!
//! These loaders are not part of the Minecraft meta manifests, so their launch
//! profile is stored next to the instance and merged into the resolved version
//! at launch time.

use std::path::Path;

use minecraft_meta_manifest::minecraft::{Argument, ArgumentType, Library};
use serde::{Deserialize, Serialize};

use crate::launcher::profile::ResolvedVersion;

/// File that marks an instance as a legacy tweaker profile.
pub const FILE: &str = "legacy-loader.json";

const LAUNCHWRAPPER_MAIN: &str = "net.minecraft.launchwrapper.Launch";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyLoaderProfile {
    /// Tweak class passed to launchwrapper, e.g. `optifine.OptiFineTweaker`.
    pub tweaker: String,
    /// Maven coordinates of the extra classpath entries.
    #[serde(default)]
    pub libraries: Vec<String>,
}

impl LegacyLoaderProfile {
    pub fn new(tweaker: impl Into<String>, libraries: Vec<String>) -> Self {
        Self {
            tweaker: tweaker.into(),
            libraries,
        }
    }
}

/// Stores the profile next to the instance so every launch can pick it up.
pub async fn write(
    instance_root: &Path,
    profile: &LegacyLoaderProfile,
) -> crate::Result<()> {
    let path = instance_root.join(FILE);
    crate::util::io::write(&path, serde_json::to_vec_pretty(profile)?).await?;
    tracing::info!(
        path = %path.display(),
        tweaker = %profile.tweaker,
        "wrote legacy loader profile"
    );
    Ok(())
}

fn game_arguments(version: &mut ResolvedVersion) -> &mut Vec<Argument> {
    let arguments = version.arguments.get_or_insert_with(Default::default);
    arguments.entry(ArgumentType::Game).or_default()
}

/// Merges the stored profile into a resolved version, when the instance has one.
pub fn apply(
    version_info: &mut ResolvedVersion,
    instance_root: &Path,
) -> crate::Result<bool> {
    let path = instance_root.join(FILE);
    if !path.try_exists()? {
        return Ok(false);
    }
    let profile: LegacyLoaderProfile =
        serde_json::from_slice(&std::fs::read(&path)?)?;

    version_info.main_class = LAUNCHWRAPPER_MAIN.to_string();

    // Versions before 1.13 describe their arguments as one legacy string, and
    // `get_minecraft_arguments` prefers `arguments` whenever it is present, so
    // a modern-style list here would silently drop the legacy arguments.
    let has_tweaker = version_info
        .minecraft_arguments
        .as_deref()
        .map(|legacy| legacy.contains(&profile.tweaker))
        .unwrap_or(false)
        || version_info
            .arguments
            .as_ref()
            .and_then(|arguments| arguments.get(&ArgumentType::Game))
            .map(|game| {
                game.iter().any(|argument| {
                    matches!(argument, Argument::Normal(value) if value == &profile.tweaker)
                })
            })
            .unwrap_or(false);

    if !has_tweaker {
        if version_info.minecraft_arguments.is_some() {
            if let Some(legacy) = version_info.minecraft_arguments.as_mut() {
                legacy.push_str(" --tweakClass ");
                legacy.push_str(&profile.tweaker);
            }
        } else {
            let game = game_arguments(version_info);
            game.push(Argument::Normal("--tweakClass".to_string()));
            game.push(Argument::Normal(profile.tweaker.clone()));
        }
    }

    for name in &profile.libraries {
        if version_info.libraries.iter().any(|library| &library.name == name)
        {
            continue;
        }
        let library: Library = serde_json::from_value(serde_json::json!({
            "name": name,
        }))?;
        version_info.libraries.push(library);
    }

    tracing::info!(
        path = %path.display(),
        tweaker = %profile.tweaker,
        "applied legacy loader profile"
    );
    Ok(true)
}

/// Applies the profile stored in any of the candidate instance roots.
pub fn apply_from_candidates(
    version_info: &mut ResolvedVersion,
    candidates: &[std::path::PathBuf],
) -> crate::Result<bool> {
    for root in candidates {
        if apply(version_info, root)? {
            return Ok(true);
        }
    }
    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn library_deserializes_from_name_only() {
        let library: Library =
            serde_json::from_value(serde_json::json!({ "name": "com.mumfrey:liteloader:1.12.2-SNAPSHOT" }))
                .expect("a library entry only needs its maven coordinate");
        assert_eq!(library.name, "com.mumfrey:liteloader:1.12.2-SNAPSHOT");
    }

    const LEGACY_VERSION: &str = r#"{
        "id": "1.12.2",
        "assets": "1.12",
        "assetIndex": {
            "id": "1.12",
            "sha1": "1e2e6b2c6b1c3e3f0a5a2c1b0f0e0d0c0b0a0908",
            "size": 1,
            "totalSize": 1,
            "url": "https://example.invalid/1.12.json"
        },
        "downloads": {},
        "libraries": [],
        "mainClass": "net.minecraft.client.main.Main",
        "minecraftArguments": "--username ${auth_player_name} --version ${version_name}",
        "minimumLauncherVersion": 21,
        "releaseTime": "2017-09-18T08:39:46+00:00",
        "time": "2017-09-18T08:39:46+00:00",
        "type": "release"
    }"#;

    #[test]
    fn legacy_arguments_get_the_tweaker() {
        let mut version: ResolvedVersion =
            serde_json::from_str(LEGACY_VERSION).expect("a legacy version manifest");
        let root = std::env::temp_dir().join(format!(
            "alemona-legacy-loader-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(
            root.join(FILE),
            serde_json::to_vec(&LegacyLoaderProfile::new(
                "com.mumfrey.liteloader.launch.LiteLoaderTweaker",
                vec!["net.minecraft:launchwrapper:1.12".to_string()],
            ))
            .unwrap(),
        )
        .unwrap();

        assert!(apply(&mut version, &root).unwrap());
        assert_eq!(version.main_class, LAUNCHWRAPPER_MAIN);
        assert!(
            version.arguments.is_none(),
            "legacy versions must keep using minecraftArguments"
        );
        assert!(
            version
                .minecraft_arguments
                .as_deref()
                .unwrap()
                .contains("--tweakClass com.mumfrey.liteloader.launch.LiteLoaderTweaker")
        );
        assert_eq!(version.libraries.len(), 1);
        std::fs::remove_dir_all(&root).ok();
    }
    #[test]
    fn profile_round_trips() {
        let profile = LegacyLoaderProfile::new(
            "optifine.OptiFineTweaker",
            vec!["net.minecraft:launchwrapper:1.12".to_string()],
        );
        let json = serde_json::to_vec(&profile).unwrap();
        let decoded: LegacyLoaderProfile = serde_json::from_slice(&json).unwrap();
        assert_eq!(decoded.tweaker, "optifine.OptiFineTweaker");
        assert_eq!(decoded.libraries.len(), 1);
    }
}
