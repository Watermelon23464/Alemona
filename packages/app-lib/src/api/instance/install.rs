use crate::state::{JavaVersion, State};
use anyhow::Context;

pub async fn get_optimal_jre_key(
    instance_id: &str,
) -> crate::Result<Option<JavaVersion>> {
    let state = State::get().await?;
    let context =
        crate::state::instances::commands::get_instance_launch_context(
            instance_id,
            &state.pool,
        )
        .await?
        .context(format!(
            "tried to resolve a nonexistent instance {instance_id}"
        ))?;
    let (minecraft, version_index) =
        crate::launcher::resolve_minecraft_manifest(
            &context.applied_content_set.game_version,
            &state,
        )
        .await?;
    let version = &minecraft.versions[version_index];
    let loader_version = crate::launcher::get_loader_version_from_profile(
        &context.applied_content_set.game_version,
        context.applied_content_set.loader,
        context.applied_content_set.loader_version.as_deref(),
        &state.directories,
    )
    .await?;
    let version_info = crate::launcher::download::download_version_info(
        &state,
        version,
        loader_version.as_ref(),
        None,
        None,
        None,
    )
    .await?;

    crate::launcher::get_java_version_from_launch_context(
        &context,
        &version_info,
        &state.pool,
    )
    .await
}

const BMCLAPI: &str = "https://bmclapi2.bangbang93.com";
const DOWNLOAD_ATTEMPTS: u32 = 3;

/// Fetches a URL, retrying a few times because the mirrors drop connections.
async fn download_bytes(
    state: &State,
    url: &str,
) -> crate::Result<bytes::Bytes> {
    let mut last_error = None;
    for attempt in 1..=DOWNLOAD_ATTEMPTS {
        match crate::util::fetch::fetch(
            url,
            None,
            None,
            None,
            &state.api_semaphore,
            &state.pool,
        )
        .await
        {
            Ok(bytes) => {
                tracing::info!(url, bytes = bytes.len(), "downloaded loader file");
                return Ok(bytes);
            }
            Err(error) => {
                tracing::warn!(url, attempt, %error, "loader file download failed");
                last_error = Some(error);
                tokio::time::sleep(std::time::Duration::from_millis(
                    400 * attempt as u64,
                ))
                .await;
            }
        }
    }
    Err(last_error.unwrap_or_else(|| {
        crate::ErrorKind::InputError(format!("could not download {url}")).into()
    }))
}

/// Downloads the first URL that answers, so callers can pass fallbacks.
async fn download_first_available(
    state: &State,
    urls: &[String],
) -> crate::Result<bytes::Bytes> {
    let mut last_error = None;
    for url in urls {
        match download_bytes(state, url).await {
            Ok(bytes) => return Ok(bytes),
            Err(error) => last_error = Some(error),
        }
    }
    Err(last_error.unwrap_or_else(|| {
        crate::ErrorKind::InputError("no download candidate worked".to_string())
            .into()
    }))
}

struct TweakerLibrary {
    /// Maven coordinate, used for the classpath entry.
    name: String,
    /// Mirrors to try, in order, when the file is missing.
    urls: Vec<String>,
    /// Path inside the shared `libraries` directory.
    path: String,
}

async fn ensure_library(
    state: &State,
    library: &TweakerLibrary,
) -> crate::Result<()> {
    let path = state.directories.libraries_dir().join(&library.path);
    if path.try_exists()? && std::fs::metadata(&path)?.len() > 0 {
        tracing::info!(?path, "loader library already present");
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        crate::util::io::create_dir_all(parent).await?;
    }
    let bytes = download_first_available(state, &library.urls).await?;
    crate::util::io::write(&path, bytes.as_ref()).await?;
    tracing::info!(?path, bytes = bytes.len(), "wrote loader library");
    Ok(())
}

/// Stores the tweaker profile next to the instance and makes sure its jars
/// are present in the shared `libraries` directory.
async fn apply_tweaker_profile(
    instance_id: &str,
    tweaker: &str,
    libraries: &[TweakerLibrary],
) -> crate::Result<String> {
    let state = State::get().await?;
    for library in libraries {
        ensure_library(&state, library).await?;
    }

    let instance_root = super::paths::get_full_path(instance_id).await?;
    let profile = crate::launcher::legacy_loader::LegacyLoaderProfile::new(
        tweaker,
        libraries
            .iter()
            .map(|library| library.name.clone())
            .collect(),
    );
    crate::launcher::legacy_loader::write(&instance_root, &profile).await?;

    let instance_name = instance_root
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_default();
    tracing::info!(
        instance = %instance_name,
        tweaker,
        "legacy loader profile installed"
    );
    Ok(instance_name)
}
fn optifine_download_urls(
    mc_version: &str,
    kind: &str,
    patch: &str,
) -> Vec<String> {
    let mut urls = vec![format!(
        "{BMCLAPI}/optifine/{mc_version}/{kind}/{patch}"
    )];
    if let Some((head, tail)) = kind.rsplit_once('_') {
        urls.push(format!(
            "{BMCLAPI}/optifine/{mc_version}/{head}/{tail}_{patch}"
        ));
    }
    urls
}

/// Downloads an OptiFine build into the instance's `mods` directory.
///
/// OptiFine is installed as a Forge mod, so a Forge instance is required for
/// every build that has a matching Forge version.
pub async fn install_optifine(
    instance_id: &str,
    mc_version: &str,
    kind: &str,
    patch: &str,
) -> crate::Result<String> {
    let state = State::get().await?;
    let instance_root = super::paths::get_full_path(instance_id).await?;
    let mods_dir = instance_root.join("mods");
    crate::util::io::create_dir_all(&mods_dir).await?;

    let bytes = download_first_available(
        &state,
        &optifine_download_urls(mc_version, kind, patch),
    )
    .await?;

    let filename = format!("OptiFine_{mc_version}_{kind}_{patch}.jar");
    let path = mods_dir.join(&filename);
    crate::util::io::write(&path, bytes.as_ref()).await?;
    tracing::info!(?path, bytes = bytes.len(), "installed OptiFine mod jar");

    Ok(filename)
}

fn optifine_libraries(
    mc_version: &str,
    kind: &str,
    patch: &str,
) -> Vec<TweakerLibrary> {
    let version = format!("{mc_version}_{kind}_{patch}");
    vec![
        TweakerLibrary {
            name: "net.minecraft:launchwrapper:1.12".to_string(),
            urls: vec![format!(
                "{BMCLAPI}/maven/net/minecraft/launchwrapper/1.12/launchwrapper-1.12.jar"
            )],
            path: "net/minecraft/launchwrapper/1.12/launchwrapper-1.12.jar"
                .to_string(),
        },
        TweakerLibrary {
            name: format!("optifine:OptiFine:{version}"),
            urls: optifine_download_urls(mc_version, kind, patch),
            path: format!("optifine/OptiFine/{version}/OptiFine-{version}.jar"),
        },
    ]
}

/// Installs OptiFine as a standalone launchwrapper tweaker profile.
///
/// Only used for builds that cannot be combined with Forge (legacy versions);
/// modern builds are installed as a Forge mod instead.
pub async fn install_optifine_standalone(
    instance_id: &str,
    mc_version: &str,
    kind: &str,
    patch: &str,
) -> crate::Result<()> {
    apply_tweaker_profile(
        instance_id,
        "optifine.OptiFineTweaker",
        &optifine_libraries(mc_version, kind, patch),
    )
    .await?;
    Ok(())
}

/// Installs LiteLoader as a launchwrapper tweaker profile.
pub async fn install_liteloader(
    instance_id: &str,
    version: &str,
    tweaker: &str,
) -> crate::Result<()> {
    let libraries = vec![
        TweakerLibrary {
            name: "net.minecraft:launchwrapper:1.12".to_string(),
            urls: vec![format!(
                "{BMCLAPI}/maven/net/minecraft/launchwrapper/1.12/launchwrapper-1.12.jar"
            )],
            path: "net/minecraft/launchwrapper/1.12/launchwrapper-1.12.jar"
                .to_string(),
        },
        TweakerLibrary {
            name: "org.ow2.asm:asm-all:5.2".to_string(),
            urls: vec![format!(
                "{BMCLAPI}/maven/org/ow2/asm/asm-all/5.2/asm-all-5.2.jar"
            )],
            path: "org/ow2/asm/asm-all/5.2/asm-all-5.2.jar".to_string(),
        },
        TweakerLibrary {
            name: format!("com.mumfrey:liteloader:{version}"),
            urls: vec![format!(
                "{BMCLAPI}/maven/com/mumfrey/liteloader/{version}/liteloader-{version}.jar"
            )],
            path: format!(
                "com/mumfrey/liteloader/{version}/liteloader-{version}.jar"
            ),
        },
    ];
    apply_tweaker_profile(instance_id, tweaker, &libraries).await?;
    Ok(())
}
