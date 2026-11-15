//! Theseus directory information
pub(crate) mod move_app_directory;
use crate::event::emit::{emit_loading, init_loading};
use crate::state::{Settings, State};
use crate::util::fetch::IoSemaphore;
use crate::{ErrorKind, LoadingBarType};
use anyhow::Context;
use std::path::{Path, PathBuf};
use tokio::fs;

pub const CACHES_FOLDER_NAME: &str = "caches";
pub const LAUNCHER_LOGS_FOLDER_NAME: &str = "launcher_logs";
pub const INSTANCES_FOLDER_NAME: &str = "versions";
pub const METADATA_FOLDER_NAME: &str = "meta";
pub const LIBRARIES_FOLDER_NAME: &str = "libraries";
pub const ASSETS_FOLDER_NAME: &str = "assets";
pub const LEGACY_ASSETS_FOLDER_NAME: &str = "resources";
pub const SYNCED_OPTIONS_FOLDER_NAME: &str = "synced-options";
pub const STORE_FOLDER_NAME: &str = "store";

#[derive(Clone, Debug)]
pub struct DirectoryInfo {
    pub settings_dir: PathBuf, // Base settings directory- app database
    pub config_dir: PathBuf, // Base config directory- instances, minecraft downloads, etc. Changeable as a setting.
    pub app_identifier: String,
}

impl DirectoryInfo {
    pub fn global_handle_if_ready() -> Option<Self> {
        State::get_if_initialized().map(|state| state.directories.clone())
    }

    pub fn get_initial_settings_dir(&self) -> Option<PathBuf> {
        Self::initial_settings_dir_path(&self.app_identifier)
    }

    // Get the settings directory
    // init() is not needed for this function
    pub fn initial_settings_dir_path(app_identifier: &str) -> Option<PathBuf> {
        Self::env_path("THESEUS_CONFIG_DIR")
            .or_else(|| Some(dirs::data_dir()?.join(app_identifier)))
    }

    /// Game directory used when the user has not picked one.
    ///
    /// Minecraft files live in the standard `.minecraft` directory so that the
    /// layout stays compatible with PCL / HMCL.
    pub fn default_config_dir(app_identifier: &str) -> Option<PathBuf> {
        Self::env_path("THESEUS_DEFAULT_DIR")
            .or_else(|| dirs::data_dir().map(|dir| dir.join(".minecraft")))
            .or_else(|| Self::initial_settings_dir_path(app_identifier))
    }

    /// Get all paths needed for Theseus to operate properly
    #[tracing::instrument]
    pub async fn init(
        config_dir: Option<String>,
        app_identifier: &str,
    ) -> crate::Result<Self> {
        let settings_dir = Self::initial_settings_dir_path(app_identifier)
            .ok_or(ErrorKind::FSError(
                "could not find valid settings dir".to_string(),
            ))?;

        fs::create_dir_all(&settings_dir)
            .await
            .context(ErrorKind::FSError(
                "error creating Theseus config directory".to_string(),
            ))?;

        let config_dir = match config_dir {
            Some(dir) => PathBuf::from(dir),
            None => Self::default_config_dir(app_identifier).ok_or(
                ErrorKind::FSError(
                    "could not find a default game directory".to_string(),
                ),
            )?,
        };

        fs::create_dir_all(&config_dir)
            .await
            .context(ErrorKind::FSError(
                "error creating the game directory".to_string(),
            ))?;

        tracing::info!(
            settings_dir = %settings_dir.display(),
            config_dir = %config_dir.display(),
            "Resolved launcher directories"
        );

        let directories = Self {
            settings_dir,
            config_dir,
            app_identifier: app_identifier.to_owned(),
        };
        directories.migrate_shared_layout().await?;
        Ok(directories)
    }

    /// Moves game files that used to live inside `meta/` to the top level of the
    /// game directory, so that other launchers sharing the root (PCL / HMCL / the
    /// vanilla launcher) reuse the same libraries and assets.
    pub async fn migrate_shared_layout(&self) -> crate::Result<()> {
        let meta = self.metadata_dir();
        for (from, to) in [
            (meta.join(LIBRARIES_FOLDER_NAME), self.libraries_dir()),
            (meta.join(ASSETS_FOLDER_NAME), self.assets_dir()),
            (meta.join(LEGACY_ASSETS_FOLDER_NAME), self.legacy_assets_dir()),
        ] {
            if !fs::try_exists(&from).await? {
                continue;
            }
            if fs::try_exists(&to).await? {
                tracing::warn!(
                    from = %from.display(),
                    to = %to.display(),
                    "The old and the new location both exist; keeping the old files in place"
                );
                continue;
            }
            if let Some(parent) = to.parent() {
                fs::create_dir_all(parent).await?;
            }
            match fs::rename(&from, &to).await {
                Ok(()) => tracing::info!(
                    from = %from.display(),
                    to = %to.display(),
                    "Moved shared Minecraft files to the top level"
                ),
                Err(error) => tracing::warn!(
                    from = %from.display(),
                    to = %to.display(),
                    "Could not move shared Minecraft files: {error}"
                ),
            }
        }
        self.migrate_instances_layout().await
    }

    /// Older builds kept instances in `profiles/`; PCL and HMCL expect `versions/`.
    async fn migrate_instances_layout(&self) -> crate::Result<()> {
        let legacy = self.config_dir.join("profiles");
        if !fs::try_exists(&legacy).await? {
            return Ok(());
        }
        let target = self.instances_dir();
        fs::create_dir_all(&target).await?;
        let mut entries = fs::read_dir(&legacy).await?;
        let mut moved = 0_usize;
        while let Some(entry) = entries.next_entry().await? {
            let to = target.join(entry.file_name());
            if move_entry(&entry.path(), &to).await? {
                moved += 1;
            }
        }
        if moved > 0 {
            tracing::info!(
                count = moved,
                "Moved instances from profiles to versions"
            );
        }
        Ok(())
    }

    pub fn store_dir(&self) -> PathBuf {
        self.config_dir.join(STORE_FOLDER_NAME)
    }

    pub fn content_store_dir(&self) -> PathBuf {
        self.store_dir().join("content")
    }

    pub fn store_staging_dir(&self) -> PathBuf {
        self.store_dir().join("staging")
    }

    /// Get the Minecraft instance metadata directory
    #[inline]
    pub fn metadata_dir(&self) -> PathBuf {
        self.config_dir.join(METADATA_FOLDER_NAME)
    }

    pub fn install_backups_dir(&self) -> PathBuf {
        self.metadata_dir().join("install_job_backups")
    }

    /// Get the Minecraft java versions metadata directory
    #[inline]
    pub fn java_versions_dir(&self) -> PathBuf {
        self.metadata_dir().join("java_versions")
    }

    /// Get the Minecraft versions metadata directory
    #[inline]
    pub fn versions_dir(&self) -> PathBuf {
        self.metadata_dir().join("versions")
    }

    /// Get the metadata directory for a given version
    #[inline]
    pub fn version_dir(&self, version: &str) -> PathBuf {
        self.versions_dir().join(version)
    }

    /// Get the shared Minecraft libraries directory
    ///
    /// Shared with the other launchers using this root (PCL / HMCL / the vanilla
    /// launcher), which is why it lives at the top level of the game directory.
    #[inline]
    pub fn libraries_dir(&self) -> PathBuf {
        self.config_dir.join(LIBRARIES_FOLDER_NAME)
    }

    /// Get the shared Minecraft assets directory
    #[inline]
    pub fn assets_dir(&self) -> PathBuf {
        self.config_dir.join(ASSETS_FOLDER_NAME)
    }

    /// Get the assets index directory
    #[inline]
    pub fn assets_index_dir(&self) -> PathBuf {
        self.assets_dir().join("indexes")
    }

    /// Get the assets objects directory
    #[inline]
    pub fn objects_dir(&self) -> PathBuf {
        self.assets_dir().join("objects")
    }

    /// Get the directory for a specific object
    #[inline]
    pub fn object_dir(&self, hash: &str) -> PathBuf {
        self.objects_dir().join(&hash[..2]).join(hash)
    }

    /// Get the Minecraft log config's directory
    #[inline]
    pub fn log_configs_dir(&self) -> PathBuf {
        self.metadata_dir().join("log_configs")
    }

    /// Get the legacy Minecraft assets directory
    #[inline]
    pub fn legacy_assets_dir(&self) -> PathBuf {
        self.config_dir.join(LEGACY_ASSETS_FOLDER_NAME)
    }

    /// Get the Minecraft legacy assets metadata directory
    #[inline]
    pub fn natives_dir(&self) -> PathBuf {
        self.metadata_dir().join("natives")
    }

    /// Get the natives directory for a version of Minecraft
    #[inline]
    pub fn version_natives_dir(&self, version: &str) -> PathBuf {
        self.natives_dir().join(version)
    }

    /// Get the directory containing instance icons
    #[inline]
    pub fn icon_dir(&self) -> PathBuf {
        self.config_dir.join("icons")
    }

    /// Get the instances directory
    #[inline]
    pub fn instances_dir(&self) -> PathBuf {
        self.config_dir.join(INSTANCES_FOLDER_NAME)
    }

    #[inline]
    pub fn synced_options_dir(&self) -> PathBuf {
        self.config_dir.join(SYNCED_OPTIONS_FOLDER_NAME)
    }

    /// Gets the logs dir for a given instance path
    #[inline]
    pub fn instance_logs_dir(&self, instance_path: &str) -> PathBuf {
        self.instances_dir().join(instance_path).join("logs")
    }

    /// Gets the crash reports dir for a given instance path
    #[inline]
    pub fn crash_reports_dir(&self, instance_path: &str) -> PathBuf {
        self.instances_dir()
            .join(instance_path)
            .join("crash-reports")
    }

    #[inline]
    pub fn launcher_logs_dir(&self) -> Option<PathBuf> {
        self.get_initial_settings_dir()
            .map(|d| d.join(LAUNCHER_LOGS_FOLDER_NAME))
    }

    #[inline]
    pub fn launcher_logs_dir_path(app_identifier: &str) -> Option<PathBuf> {
        Self::initial_settings_dir_path(app_identifier)
            .map(|d| d.join(LAUNCHER_LOGS_FOLDER_NAME))
    }

    /// Get the cache directory for Theseus
    #[inline]
    pub fn caches_dir(&self) -> PathBuf {
        self.config_dir.join(CACHES_FOLDER_NAME)
    }

    /// Get path from environment variable
    #[inline]
    fn env_path(name: &str) -> Option<PathBuf> {
        std::env::var_os(name).map(PathBuf::from)
    }

    #[tracing::instrument(skip(settings, pool, _io_semaphore))]
    pub async fn move_launcher_directory(
        settings: &mut Settings,
        pool: &sqlx::SqlitePool,
        _io_semaphore: &IoSemaphore,
        app_identifier: &str,
    ) -> crate::Result<()> {
        Self::try_move_launcher_directory(settings, pool, app_identifier)
            .await
            .context(ErrorKind::DirectoryMoveFailed)
    }

    async fn try_move_launcher_directory(
        settings: &mut Settings,
        pool: &sqlx::SqlitePool,
        app_identifier: &str,
    ) -> crate::Result<()> {
        let initial = DirectoryInfo::initial_settings_dir_path(app_identifier)
            .ok_or(ErrorKind::FSError(
                "could not find the app directory".to_string(),
            ))?;
        let default_dir = DirectoryInfo::default_config_dir(app_identifier)
            .unwrap_or_else(|| initial.clone());
        let stored = settings
            .custom_dir
            .as_deref()
            .filter(|dir| !same_dir(Path::new(dir), &initial));
        tracing::info!(
            legacy = %initial.display(),
            default = %default_dir.display(),
            stored = ?settings.custom_dir,
            previous = ?settings.prev_custom_dir,
            "Resolving the game directory"
        );
        let mut destination = stored
            .map(PathBuf::from)
            .unwrap_or_else(|| default_dir.clone());
        let mut previous = settings.prev_custom_dir.as_ref().map(PathBuf::from);
        if previous.is_none()
            && stored.is_none()
            && !same_dir(&initial, &destination)
            && (fs::try_exists(initial.join(INSTANCES_FOLDER_NAME)).await?
                || fs::try_exists(initial.join("profiles")).await?)
        {
            previous = Some(initial.clone());
        }
        let previous_root = match &previous {
            Some(previous) => Some(fs::canonicalize(previous).await?),
            None => None,
        };
        fs::create_dir_all(&destination).await?;
        let destination_root = fs::canonicalize(&destination).await?;
        let mut moving = previous_root
            .as_ref()
            .is_some_and(|root| root != &destination_root);
        let pending_move =
            super::content_store::setting(pool, "store_directory_move")
                .await?
                .filter(|checkpoint| !checkpoint.is_empty())
                .map(|checkpoint| {
                    serde_json::from_str::<(PathBuf, PathBuf)>(&checkpoint)
                })
                .transpose()?;
        let settings_root = fs::canonicalize(&initial).await?;
        let mut locked_roots = std::collections::HashSet::from([settings_root]);
        let mut move_locks = Vec::new();
        let pending_roots = pending_move
            .as_ref()
            .into_iter()
            .flat_map(|(from, to)| [from, to]);
        for root in previous
            .iter()
            .chain(std::iter::once(&destination))
            .chain(pending_roots)
        {
            let root = fs::canonicalize(root).await?;
            if locked_roots.insert(root.clone()) {
                move_locks.push(
                    super::content_store::ContentStore::lock_process(&root)
                        .await?,
                );
            }
        }
        if let Some(previous) = &previous
            && moving
        {
            let loading = init_loading(
                LoadingBarType::DirectoryMove {
                    old: previous.to_string_lossy().into_owned(),
                    new: destination.to_string_lossy().into_owned(),
                },
                100.0,
                "Moving launcher directory",
            )
            .await?;
            match move_app_directory::move_app_directory(
                previous,
                &destination,
                pool,
            )
            .await
            {
                Ok(()) => emit_loading(&loading, 100.0, None)?,
                Err(error) => {
                    tracing::warn!(
                        %error,
                        "Could not migrate the game directory; keeping the previous location"
                    );
                    destination = previous.clone();
                    moving = false;
                }
            }
        }
        if !moving {
            move_app_directory::resume_completed_move(&destination, pool)
                .await?;
        }
        settings.custom_dir = Some(destination.to_string_lossy().into_owned());
        settings.prev_custom_dir.clone_from(&settings.custom_dir);
        settings.update(pool).await?;
        if let Some(previous) = &previous
            && moving
        {
            move_app_directory::finish_app_directory_move(
                previous,
                &destination,
                pool,
            )
            .await?;
        }
        drop(move_locks);
        Ok(())
    }
}

/// Compares two directories, tolerating different separators or casing.
fn same_dir(left: &Path, right: &Path) -> bool {
    if left == right {
        return true;
    }
    match (dunce::canonicalize(left), dunce::canonicalize(right)) {
        (Ok(left), Ok(right)) => left == right,
        _ => false,
    }
}

/// Moves `from` to `to`, merging into `to` when it is an existing directory.
///
/// Returns whether anything was moved. Existing entries are kept as they are.
async fn move_entry(from: &Path, to: &Path) -> crate::Result<bool> {
    if !fs::try_exists(from).await? {
        return Ok(false);
    }
    if !fs::try_exists(to).await? {
        if let Some(parent) = to.parent() {
            fs::create_dir_all(parent).await?;
        }
        return Ok(fs::rename(from, to).await.is_ok());
    }

    let from_metadata = fs::symlink_metadata(from).await?;
    let to_metadata = fs::symlink_metadata(to).await?;
    if !from_metadata.is_dir() || !to_metadata.is_dir() {
        tracing::warn!(
            from = %from.display(),
            to = %to.display(),
            "Keeping both copies of a file that already exists in the new layout"
        );
        return Ok(false);
    }

    let mut moved = false;
    let mut entries = fs::read_dir(from).await?;
    while let Some(entry) = entries.next_entry().await? {
        let child_to = to.join(entry.file_name());
        moved |= Box::pin(move_entry(&entry.path(), &child_to)).await?;
    }
    if fs::read_dir(from).await?.next_entry().await?.is_none() {
        let _ = fs::remove_dir(from).await;
    }
    Ok(moved)
}
