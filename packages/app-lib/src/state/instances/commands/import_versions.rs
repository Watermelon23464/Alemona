//! Registers the Minecraft versions other launchers installed into this game
//! directory (PCL, HMCL, the vanilla launcher), so that they show up in the
//! instance library and can be launched from here as well.

use crate::state::instances::adapters::sqlite::{
    content_rows, instance_rows,
};
use crate::state::instances::{
    ContentSet, ContentSetStatus, ContentSourceKind, Instance,
    InstanceLaunchOverrides, InstanceLink,
};
use crate::state::{
    InstanceInstallStage, LauncherFeatureVersion, ModLoader, ReleaseChannel,
    State,
};
use crate::util::io;
use chrono::Utc;
use serde_json::Value;
use std::collections::HashSet;
use std::path::Path;
use tokio::fs;
use uuid::Uuid;

/// Vanilla main class, shared by every version that has no mod loader attached.
const VANILLA_MAIN_CLASS: &str = "net.minecraft.client.main.Main";

/// Longest `inheritsFrom` chain that is followed when reading a version.
const MAX_CHAIN_LENGTH: usize = 8;

/// Libraries that mark a version as something this launcher cannot rebuild.
const UNSUPPORTED_LIBRARIES: [&str; 3] = ["optifine", "liteloader", "litemod"];

struct ExternalVersion {
    game_version: String,
    loader: ModLoader,
    loader_version: Option<String>,
}

/// Imports every version directory that has no instance yet.
pub(crate) async fn import_external_versions(
    state: &State,
) -> crate::Result<usize> {
    let root = state.directories.instances_dir();
    if !fs::try_exists(&root).await? {
        return Ok(0);
    }

    let mut known = instance_rows::list_instances(&state.pool)
        .await?
        .into_iter()
        .map(|instance| instance.path)
        .collect::<HashSet<_>>();

    let mut imported = Vec::new();
    let mut entries = fs::read_dir(&root).await?;
    while let Some(entry) = entries.next_entry().await? {
        if !entry.file_type().await?.is_dir() {
            continue;
        }
        let Some(name) = entry.file_name().to_str().map(str::to_owned) else {
            continue;
        };
        if known.contains(&name) {
            continue;
        }
        let Some(external) = resolve_external_version(&root, &name).await else {
            continue;
        };

        match register(state, &name, &external).await {
            Ok(instance_id) => {
                known.insert(name.clone());
                tracing::info!(
                    version = %name,
                    game_version = %external.game_version,
                    loader = external.loader.as_str(),
                    loader_version = external.loader_version.as_deref().unwrap_or(""),
                    "Imported a version installed by another launcher"
                );
                imported.push(instance_id);
            }
            Err(error) => tracing::warn!(
                version = %name,
                "Could not import a version installed by another launcher: {error}"
            ),
        }
    }

    for instance_id in &imported {
        if let Err(error) = crate::event::emit::emit_instance(
            instance_id,
            crate::event::InstancePayloadType::Created,
        )
        .await
        {
            tracing::warn!("Could not emit the imported instance: {error}");
        }
    }

    Ok(imported.len())
}

/// Reads `versions/<name>/<name>.json` and follows its `inheritsFrom` chain.
async fn read_chain(versions: &Path, name: &str) -> Option<Vec<Value>> {
    let mut chain = Vec::new();
    let mut current = Some(name.to_string());
    let mut seen = HashSet::new();
    while let Some(id) = current {
        if !seen.insert(id.clone()) || chain.len() >= MAX_CHAIN_LENGTH {
            break;
        }
        let path = versions.join(&id).join(format!("{id}.json"));
        let Ok(bytes) = fs::read(&path).await else {
            break;
        };
        let Ok(value) = serde_json::from_slice::<Value>(bytes.as_slice())
        else {
            break;
        };
        current = value
            .get("inheritsFrom")
            .and_then(Value::as_str)
            .map(str::to_owned);
        chain.push(value);
    }
    (!chain.is_empty()).then_some(chain)
}

async fn resolve_external_version(
    versions: &Path,
    name: &str,
) -> Option<ExternalVersion> {
    let chain = read_chain(versions, name).await?;
    let manifest = chain.first()?;

    let game_version = manifest
        .get("inheritsFrom")
        .and_then(Value::as_str)
        .or_else(|| manifest.get("clientVersion").and_then(Value::as_str))
        .map(str::to_owned)
        .or_else(|| root_version_id(&chain))?;

    let main_class = chain
        .iter()
        .find_map(|entry| entry.get("mainClass").and_then(Value::as_str));
    let (loader, loader_version) = detect_loader(&chain)?;
    if loader == ModLoader::Vanilla
        && main_class.is_some_and(|main_class| main_class != VANILLA_MAIN_CLASS)
    {
        tracing::debug!(
            version = %name,
            main_class = main_class.unwrap_or_default(),
            "Skipping a version that runs a launcher-specific main class"
        );
        return None;
    }

    Some(ExternalVersion {
        game_version,
        loader,
        loader_version,
    })
}

fn root_version_id(chain: &[Value]) -> Option<String> {
    chain
        .last()?
        .get("id")
        .and_then(Value::as_str)
        .map(str::to_owned)
}

/// Detects the mod loader from the maven coordinates of every library in the
/// chain. Flattened manifests list the loader libraries themselves, while
/// HMCL-style manifests keep them in the child entry.
fn detect_loader(chain: &[Value]) -> Option<(ModLoader, Option<String>)> {
    let mut unsupported = false;
    for manifest in chain {
        let Some(libraries) = manifest.get("libraries").and_then(Value::as_array)
        else {
            continue;
        };
        for library in libraries {
            let Some(name) = library.get("name").and_then(Value::as_str) else {
                continue;
            };
            let lowercase = name.to_lowercase();
            if UNSUPPORTED_LIBRARIES
                .iter()
                .any(|marker| lowercase.contains(marker))
            {
                unsupported = true;
            }

            let mut parts = name.split(':');
            let group = parts.next().unwrap_or_default();
            let artifact = parts.next().unwrap_or_default();
            let version = parts.next().unwrap_or_default();
            let loaded = match (group, artifact) {
                ("net.fabricmc", "fabric-loader") => {
                    Some((ModLoader::Fabric, version))
                }
                ("org.quiltmc", "quilt-loader") => {
                    Some((ModLoader::Quilt, version))
                }
                ("net.minecraftforge", "forge") => Some((
                    ModLoader::Forge,
                    version.rsplit_once('-').map_or(version, |(_, v)| v),
                )),
                ("net.neoforged", "neoforge")
                | ("net.neoforged", "forge") => {
                    Some((ModLoader::NeoForge, version))
                }
                _ => None,
            };

            if let Some((loader, version)) = loaded {
                return Some((loader, Some(version.to_string())));
            }
        }
    }

    (!unsupported).then_some((ModLoader::Vanilla, None))
}

async fn register(
    state: &State,
    name: &str,
    external: &ExternalVersion,
) -> crate::Result<String> {
    let now = Utc::now();
    let instance_id = format!("local:{}", Uuid::new_v4());
    let content_set_id = format!("content-set:{}", Uuid::new_v4());
    let directory = state.directories.instances_dir().join(name);
    let protocol_version = crate::launcher::read_protocol_version_from_jar(
        directory.join(format!("{name}.jar")),
    )
    .await
    .ok()
    .flatten();
    let instance = Instance {
        id: instance_id.clone(),
        path: name.to_string(),
        applied_content_set_id: Some(content_set_id.clone()),
        install_stage: InstanceInstallStage::Installed,
        launcher_feature_version: LauncherFeatureVersion::MOST_RECENT,
        update_channel: ReleaseChannel::Release,
        name: name.to_string(),
        icon_path: None,
        created: now,
        modified: now,
        last_played: None,
        submitted_time_played: 0,
        recent_time_played: 0,
    };
    let content_set = ContentSet {
        id: content_set_id,
        instance_id: instance_id.clone(),
        name: "Default".to_string(),
        source_kind: ContentSourceKind::Local,
        status: ContentSetStatus::Available,
        game_version: external.game_version.clone(),
        protocol_version,
        loader: external.loader,
        loader_version: external.loader_version.clone(),
        created: now,
        modified: now,
    };

    let mut tx = state.pool.begin_with("BEGIN IMMEDIATE").await?;
    instance_rows::insert_instance(&instance, &mut tx).await?;
    instance_rows::insert_default_instance_sync_preferences(
        &instance_id,
        &mut tx,
    )
    .await?;
    content_rows::insert_content_set(&content_set, &mut tx).await?;
    instance_rows::upsert_instance_link(
        &instance_id,
        &InstanceLink::Unmanaged,
        &mut tx,
    )
    .await?;
    instance_rows::replace_instance_groups(&instance_id, &[], &mut tx)
        .await?;
    instance_rows::upsert_instance_launch_overrides(
        &InstanceLaunchOverrides::empty(instance_id.clone()),
        &mut tx,
    )
    .await?;
    tx.commit().await?;

    crate::state::instances::watcher::watch_instance_folder(
        &instance_id,
        &instance.path,
        &state.file_watcher,
        &state.directories,
    )
    .await;

    if let Ok(full_path) = io::canonicalize(&directory) {
        tracing::info!(
            path = %full_path.display(),
            protocol_version = protocol_version.unwrap_or_default(),
            "Registered an existing version directory"
        );
    }

    Ok(instance_id)
}
