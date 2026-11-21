//! Terracotta Networking (陶瓦联机) integration.
//!
//! The launcher never ships Terracotta itself: the user is asked once whether
//! to download the official release, which then lives in the app data
//! directory.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::State;

/// Folder inside the app data directory that holds the downloaded release.
pub const FOLDER: &str = "terracotta";

/// Release asset for the current platform, or `None` when unsupported.
fn release_asset() -> Option<&'static str> {
    if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
        Some("windows-x86_64")
    } else if cfg!(all(target_os = "windows", target_arch = "aarch64")) {
        Some("windows-arm64")
    } else if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        Some("linux-x86_64")
    } else if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        Some("macos-arm64")
    } else if cfg!(all(target_os = "macos", target_arch = "x86_64")) {
        Some("macos-x86_64")
    } else {
        None
    }
}


/// Where the downloaded release lives: `<launcher dir>/AL/terracotta`.
pub fn install_dir() -> crate::Result<PathBuf> {
    let exe = std::env::current_exe()?;
    let dir = exe.parent().ok_or_else(|| {
        crate::ErrorKind::LauncherError("launcher directory not found".to_string())
    })?;
    Ok(dir.join("AL").join(FOLDER))
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TerracottaInstall {
    /// Whether an executable was found in the install directory.
    pub installed: bool,
    /// Absolute path of the install directory.
    pub dir: String,
    /// Absolute path of the executable, when installed.
    pub executable: Option<String>,
    /// Version of the downloaded release, read back from the folder name.
    pub version: Option<String>,
}

/// Releases ship as `terracotta-<version>-<platform>.exe`, so any executable
/// whose name contains "terracotta" counts as the installed binary.
fn find_executable(dir: &Path) -> Option<PathBuf> {
    let entries = std::fs::read_dir(dir).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Some(found) = find_executable(&path) {
                return Some(found);
            }
            continue;
        }
        let name = path
            .file_name()
            .map(|name| name.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        if !name.contains("terracotta") || name.ends_with(".tar.gz") {
            continue;
        }
        let is_executable = if cfg!(windows) {
            name.ends_with(".exe")
        } else {
            !name.contains('.')
        };
        if is_executable {
            return Some(path);
        }
    }
    None
}

/// Reads the release version out of the binary name, e.g. `0.4.2`.
fn read_version(executable: &Path) -> Option<String> {
    let name = executable.file_name()?.to_string_lossy().to_string();
    let mut digits = String::new();
    let mut version = String::new();
    let mut parts = 0;
    for character in name.chars() {
        if character.is_ascii_digit() {
            digits.push(character);
        } else if character == '.' && !digits.is_empty() && parts < 2 {
            version.push_str(&digits);
            version.push('.');
            digits.clear();
            parts += 1;
        } else if !digits.is_empty() {
            break;
        }
    }
    if digits.is_empty() || parts == 0 {
        return None;
    }
    version.push_str(&digits);
    Some(version)
}

/// Current install state, without touching the network.
pub async fn get_install() -> crate::Result<TerracottaInstall> {
    let dir = install_dir()?;
    let executable = find_executable(&dir);
    Ok(TerracottaInstall {
        installed: executable.is_some(),
        dir: dir.to_string_lossy().to_string(),
        version: executable.as_deref().and_then(read_version),
        executable: executable.map(|path| path.to_string_lossy().to_string()),
    })
}

#[derive(Deserialize)]
struct ReleaseAsset {
    name: String,
    browser_download_url: String,
}

#[derive(Deserialize)]
struct Release {
    tag_name: String,
    assets: Vec<ReleaseAsset>,
}

/// Downloads and unpacks the official Terracotta release for this platform.
pub async fn install() -> crate::Result<TerracottaInstall> {
    let state = State::get().await?;
    let target = release_asset().ok_or_else(|| {
        crate::ErrorKind::InputError(
            "Terracotta does not publish a build for this platform".to_string(),
        )
    })?;

    let release_bytes = crate::util::fetch::fetch(
        "https://api.github.com/repos/burningtnt/Terracotta/releases/latest",
        None,
        None,
        None,
        &state.api_semaphore,
        &state.pool,
    )
    .await?;
    let release: Release = serde_json::from_slice(release_bytes.as_ref())?;

    let asset = release
        .assets
        .iter()
        .find(|asset| {
            asset.name.contains(target) && asset.name.ends_with("-pkg.tar.gz")
        })
        .ok_or_else(|| {
            crate::ErrorKind::InputError(format!(
                "no Terracotta release asset for {target}"
            ))
        })?;

    let dir = install_dir()?;
    crate::util::io::create_dir_all(&dir).await?;

    let archive = dir.join(&asset.name);
    if !archive.try_exists()? {
        let bytes = crate::util::fetch::fetch(
            &asset.browser_download_url,
            None,
            None,
            None,
            &state.api_semaphore,
            &state.pool,
        )
        .await?;
        crate::util::io::write(&archive, bytes.as_ref()).await?;
        tracing::info!(
            path = %archive.display(),
            bytes = bytes.len(),
            "downloaded Terracotta release"
        );
    }

    let status = tokio::process::Command::new("tar")
        .arg("-xzf")
        .arg(&archive)
        .arg("-C")
        .arg(&dir)
        .status()
        .await?;
    if !status.success() {
        return Err(crate::ErrorKind::LauncherError(format!(
            "could not unpack {}",
            archive.display()
        ))
        .into());
    }

    tracing::info!(
        version = %release.tag_name,
        dir = %dir.display(),
        "unpacked Terracotta release"
    );

    get_install().await
}

static SESSION_PORT: std::sync::atomic::AtomicU16 =
    std::sync::atomic::AtomicU16::new(0);
static SESSION_CHILD: std::sync::OnceLock<
    std::sync::Mutex<Option<tokio::process::Child>>,
> = std::sync::OnceLock::new();

fn child_slot() -> &'static std::sync::Mutex<Option<tokio::process::Child>> {
    SESSION_CHILD.get_or_init(|| std::sync::Mutex::new(None))
}

/// Port of the running Terracotta HTTP service, when one is known.
pub fn session_port() -> u16 {
    SESSION_PORT.load(std::sync::atomic::Ordering::Relaxed)
}

fn base_url(port: u16) -> String {
    format!("http://127.0.0.1:{port}")
}

async fn get(port: u16, path: &str) -> crate::Result<String> {
    let response = reqwest::Client::new()
        .get(format!("{}{}", base_url(port), path))
        .header("Accept", "application/json")
        .send()
        .await?;
    Ok(response.text().await?)
}

async fn get_json(
    port: u16,
    path: &str,
) -> crate::Result<serde_json::Value> {
    let body = get(port, path).await?;
    Ok(serde_json::from_str(&body)?)
}

fn lock_file_port() -> Option<u16> {
    let path = std::env::temp_dir()
        .join("terracotta")
        .join("terracotta.lock");
    let bytes = std::fs::read(path).ok()?;
    if bytes.len() < 2 {
        return None;
    }
    Some(u16::from_be_bytes([bytes[0], bytes[1]]))
}

async fn probe(port: u16) -> bool {
    matches!(get(port, "/meta").await, Ok(body) if !body.is_empty())
}

/// Starts Terracotta when needed and returns the HTTP API port.
pub async fn launch() -> crate::Result<u16> {
    let known = session_port();
    if known != 0 && probe(known).await {
        return Ok(known);
    }

    if let Some(port) = lock_file_port() {
        if probe(port).await {
            SESSION_PORT.store(port, std::sync::atomic::Ordering::Relaxed);
            tracing::info!(port, "reusing the running Terracotta instance");
            return Ok(port);
        }
    }

    let executable = get_install().await?.executable.ok_or_else(|| {
        crate::ErrorKind::InputError(
            "Terracotta is not installed yet".to_string(),
        )
    })?;
    let executable = PathBuf::from(executable);
    let dir = install_dir()?;

    let port_file = std::env::temp_dir().join("alemona-terracotta-port.json");
    let _ = std::fs::remove_file(&port_file);

    let mut child = tokio::process::Command::new(&executable)
        .arg("--hmcl")
        .arg(&port_file)
        .current_dir(&dir)
        .spawn()?;

    let mut port = 0u16;
    for _ in 0..100 {
        if let Ok(raw) = std::fs::read(&port_file) {
            if let Ok(value) =
                serde_json::from_slice::<serde_json::Value>(&raw)
            {
                if let Some(found) = value
                    .get("port")
                    .and_then(|port| port.as_u64())
                    .map(|port| port as u16)
                {
                    port = found;
                    break;
                }
            }
        }
        if let Ok(Some(_)) = child.try_wait() {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    let _ = std::fs::remove_file(&port_file);
    if port == 0 {
        child.start_kill().ok();
        return Err(crate::ErrorKind::LauncherError(
            "Terracotta did not report its HTTP port".to_string(),
        )
        .into());
    }

    *child_slot()
        .lock()
        .map_err(|_| {
            crate::ErrorKind::LauncherError("terracotta lock".to_string())
        })? = Some(child);
    SESSION_PORT.store(port, std::sync::atomic::Ordering::Relaxed);
    tracing::info!(port, "started Terracotta");
    Ok(port)
}

/// Current room state from the Terracotta state machine.
pub async fn state() -> crate::Result<serde_json::Value> {
    let port = launch().await?;
    let value = get_json(port, "/state").await?;
    if let Some(state) = value.get("state").and_then(|state| state.as_str()) {
        LAST_STATE.with(|last| {
            let mut last = last.borrow_mut();
            if last.as_deref() != Some(state) {
                tracing::info!(state, "terracotta: state changed");
                *last = Some(state.to_string());
            }
        });
    }
    Ok(value)
}

thread_local! {
    static LAST_STATE: std::cell::RefCell<Option<String>> =
        const { std::cell::RefCell::new(None) };
}



/// Asks Terracotta to host a room.
///
/// The control endpoints answer with an empty body, so the response is not
/// parsed; the state machine reports the outcome through `/state`.
pub async fn host(player: Option<&str>) -> crate::Result<()> {
    let port = launch().await?;
    // Empty query values confuse the scanner, so only send what we have.
    let mut path = "/state/scanning".to_string();
    if let Some(player) = player.filter(|value| !value.trim().is_empty()) {
        path.push_str(&format!("?player={}", urlencode(player)));
    }
    tracing::info!(path, "terracotta: asking to host");
    get(port, &path).await?;
    Ok(())
}

/// Asks Terracotta to join a room.
pub async fn join(room: &str, player: Option<&str>) -> crate::Result<()> {
    let port = launch().await?;
    let mut path = format!("/state/guesting?room={}", urlencode(room));
    if let Some(player) = player.filter(|value| !value.trim().is_empty()) {
        path.push_str(&format!("&player={}", urlencode(player)));
    }
    tracing::info!(path, "terracotta: asking to join");
    get(port, &path).await?;
    Ok(())
}

/// Returns to the idle state.
pub async fn reset() -> crate::Result<()> {
    let port = launch().await?;
    get(port, "/state/ide").await?;
    Ok(())
}

/// Shuts the Terracotta process down.
pub async fn stop() -> crate::Result<()> {
    let port = session_port();
    if port != 0 {
        let _ = get(port, "/panic?peaceful=true").await;
        SESSION_PORT.store(0, std::sync::atomic::Ordering::Relaxed);
    }
    if let Ok(mut slot) = child_slot().lock() {
        if let Some(child) = slot.as_mut() {
            child.start_kill().ok();
        }
        *slot = None;
    }
    Ok(())
}

fn urlencode(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.'
            | b'~' => encoded.push(byte as char),
            _ => encoded.push_str(&format!("%{byte:02X}")),
        }
    }
    encoded
}
