//! Version entries in the on-disk format used by PCL and HMCL.
//!
//! Every installed instance lives in `versions/<name>/`, which is also the game
//! directory of that instance. Writing `<name>/<name>.json` and `<name>/<name>.jar`
//! lets other launchers sharing the same `.minecraft` root launch it, and lets this
//! launcher pick up the versions those launchers installed.

use crate::state::DirectoryInfo;
use crate::state::InstanceInstallStage;
use crate::state::instances::adapters::sqlite::instance_rows;
use serde_json::Value;
use std::path::Path;
use tokio::fs;

/// Marks a version directory whose version entry is written by this launcher.
///
/// Entries written by PCL or HMCL describe a setup only they can rebuild, so they
/// are left untouched.
const MANAGED_MARKER: &str = ".alemona-version";

/// Directory name of an instance inside the versions directory.
pub fn version_id(instance_path: &str) -> Option<&str> {
    Path::new(instance_path)
        .file_name()
        .and_then(|name| name.to_str())
}

/// Cache key of the launcher metadata describing an instance's game files.
pub fn version_jar_id(game_version: &str, loader_version: Option<&str>) -> String {
    match loader_version {
        Some(loader_version) => format!("{game_version}-{loader_version}"),
        None => game_version.to_string(),
    }
}

/// Writes the version entry of an installed instance.
///
/// The entry is a flattened manifest (the same shape PCL writes), so the only
/// absolute paths involved are the ones the launcher fills in from the
/// `${...}` placeholders at launch time.
pub async fn export_version_entry(
    directories: &DirectoryInfo,
    instance_path: &str,
    game_version: &str,
    loader_version: Option<&str>,
) -> crate::Result<bool> {
    let Some(version_id) = version_id(instance_path) else {
        return Ok(false);
    };
    let version_jar = version_jar_id(game_version, loader_version);
    let metadata_path = directories
        .version_dir(&version_jar)
        .join(format!("{version_jar}.json"));
    let metadata = match fs::read(&metadata_path).await {
        Ok(metadata) => metadata,
        Err(error) => {
            tracing::debug!(
                path = %metadata_path.display(),
                "Skipping the version entry export: {error}"
            );
            return Ok(false);
        }
    };

    let mut manifest: Value = serde_json::from_slice(metadata.as_slice())?;
    let Some(manifest_object) = manifest.as_object_mut() else {
        return Ok(false);
    };
    manifest_object.insert("id".to_string(), Value::String(version_id.to_string()));
    manifest_object.insert(
        "clientVersion".to_string(),
        Value::String(game_version.to_string()),
    );

    let directory = directories.instances_dir().join(instance_path);
    fs::create_dir_all(&directory).await?;

    let json_path = directory.join(format!("{version_id}.json"));
    let marker_path = directory.join(MANAGED_MARKER);
    if fs::try_exists(&json_path).await?
        && !fs::try_exists(&marker_path).await?
    {
        let Ok(existing) = fs::read(&json_path).await else {
            return Ok(false);
        };
        let Ok(existing) = serde_json::from_slice::<Value>(existing.as_slice())
        else {
            return Ok(false);
        };
        if !describes_same_setup(
            &existing,
            &manifest,
            version_id,
            game_version,
            &version_jar,
        ) {
            tracing::debug!(
                path = %json_path.display(),
                "Keeping the version entry written by another launcher"
            );
            return Ok(false);
        }
    }

    fs::write(&json_path, serde_json::to_vec_pretty(&manifest)?).await?;
    fs::write(&marker_path, b"managed by Alemona Launcher\n").await?;

    let client = directories
        .version_dir(&version_jar)
        .join(format!("{version_jar}.jar"));
    link_client(&client, &directory.join(format!("{version_id}.jar"))).await?;

    Ok(true)
}

/// Refreshes the version entry of every installed instance.
///
/// Instances installed by an older build have no entry yet, and entries written
/// by other launchers may describe a different loader version, so the entries are
/// rewritten whenever the launcher starts.
pub async fn sync_version_entries(state: &crate::State) -> crate::Result<()> {
    let mut exported = 0_usize;
    for instance in instance_rows::list_instances(&state.pool).await? {
        if instance.install_stage != InstanceInstallStage::Installed {
            continue;
        }
        let Some(context) =
            instance_rows::get_instance_launch_context(&instance.id, &state.pool)
                .await?
        else {
            continue;
        };
        let content = &context.applied_content_set;
        let exported_entry = export_version_entry(
            &state.directories,
            &instance.path,
            &content.game_version,
            content.loader_version.as_deref(),
        )
        .await;
        match exported_entry {
            Ok(true) => exported += 1,
            Ok(false) => {}
            Err(error) => tracing::warn!(
                instance_id = %instance.id,
                "Could not write the version entry for {}: {error}",
                instance.path
            ),
        }
    }
    if exported > 0 {
        tracing::info!(count = exported, "Exported version entries");
    }
    Ok(())
}

/// Whether an entry written before the marker existed still describes the same
/// launch setup, which makes rewriting it harmless.
fn describes_same_setup(
    existing: &Value,
    manifest: &Value,
    version_id: &str,
    game_version: &str,
    version_jar: &str,
) -> bool {
    let existing_id = existing.get("id").and_then(Value::as_str);
    // Entries written by an older build of this launcher kept the metadata id
    // (the game and loader version) instead of the version directory name.
    existing_id == Some(version_jar)
        || (existing_id == Some(version_id)
            && existing.get("clientVersion").and_then(Value::as_str)
                == Some(game_version)
            && existing.get("mainClass") == manifest.get("mainClass"))
}

async fn link_client(client: &Path, target: &Path) -> crate::Result<()> {
    let Ok(source_metadata) = fs::metadata(client).await else {
        return Ok(());
    };
    if let Ok(target_metadata) = fs::metadata(target).await
        && target_metadata.len() == source_metadata.len()
    {
        return Ok(());
    }
    if fs::try_exists(target).await? {
        fs::remove_file(target).await?;
    }
    if fs::hard_link(client, target).await.is_ok() {
        return Ok(());
    }
    fs::copy(client, target).await?;
    Ok(())
}
