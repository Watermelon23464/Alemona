//! Bookkeeping for Minecraft accounts.
//!
//! theseus stores the credentials themselves; this module remembers which kind
//! of account they are (Microsoft, offline, LittleSkin or another Yggdrasil
//! provider), the provider URL for third-party accounts, and how often each
//! account has been used so the launcher can show a usage share.
//!
//! Everything lives in `AL/cache/accounts.json` next to the launcher binary.

use std::path::PathBuf;

use anyhow::{Context, anyhow};
use base64::Engine;
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::state::{Credentials, MinecraftProfile, State};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccountKind {
    Microsoft,
    Offline,
    #[serde(rename = "littleskin", alias = "little_skin")]
    LittleSkin,
    Yggdrasil,
}

impl Default for AccountKind {
    fn default() -> Self {
        Self::Microsoft
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AccountMeta {
    pub uuid: String,
    #[serde(default)]
    pub kind: AccountKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_root: Option<String>,
    #[serde(default)]
    pub usage: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_used: Option<i64>,
}

fn al_dir() -> crate::Result<PathBuf> {
    let exe = std::env::current_exe()?;
    Ok(exe
        .parent()
        .ok_or_else(|| anyhow!("launcher directory not found"))?
        .join("AL"))
}

fn cache_dir() -> crate::Result<PathBuf> {
    let dir = al_dir()?.join("cache");
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn cache_file() -> crate::Result<PathBuf> {
    Ok(cache_dir()?.join("accounts.json"))
}

fn libraries_dir() -> crate::Result<PathBuf> {
    let dir = al_dir()?.join("libraries");
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub fn load_all() -> Vec<AccountMeta> {
    (|| -> crate::Result<Vec<AccountMeta>> {
        let path = cache_file()?;
        if !path.exists() {
            return Ok(Vec::new());
        }
        let raw = std::fs::read(path)?;
        Ok(serde_json::from_slice(&raw).unwrap_or_default())
    })()
    .unwrap_or_default()
}

fn save_all(accounts: &[AccountMeta]) -> crate::Result<()> {
    let path = cache_file()?;
    std::fs::write(path, serde_json::to_vec_pretty(accounts)?)?;
    Ok(())
}

pub fn get(uuid: Uuid) -> Option<AccountMeta> {
    let key = uuid.to_string();
    load_all().into_iter().find(|entry| entry.uuid == key)
}

/// Creates or updates the metadata of a single account.
pub fn upsert(mut meta: AccountMeta) -> crate::Result<AccountMeta> {
    let mut accounts = load_all();
    if let Some(existing) = accounts.iter_mut().find(|entry| entry.uuid == meta.uuid) {
        meta.usage = existing.usage.max(meta.usage);
        meta.last_used = meta.last_used.or(existing.last_used);
        *existing = meta.clone();
    } else {
        accounts.push(meta.clone());
    }
    save_all(&accounts)?;
    Ok(meta)
}

/// Counts one use of an account and returns its updated metadata.
pub fn record_usage(uuid: Uuid) -> crate::Result<AccountMeta> {
    let key = uuid.to_string();
    let mut accounts = load_all();
    let index = accounts.iter().position(|entry| entry.uuid == key);
    let meta = if let Some(index) = index {
        let entry = &mut accounts[index];
        entry.usage += 1;
        entry.last_used = Some(Utc::now().timestamp());
        entry.clone()
    } else {
        let entry = AccountMeta {
            uuid: key,
            usage: 1,
            last_used: Some(Utc::now().timestamp()),
            ..AccountMeta::default()
        };
        accounts.push(entry.clone());
        entry
    };
    save_all(&accounts)?;
    Ok(meta)
}

pub fn forget(uuid: Uuid) -> crate::Result<()> {
    let key = uuid.to_string();
    let accounts: Vec<AccountMeta> = load_all()
        .into_iter()
        .filter(|entry| entry.uuid != key)
        .collect();
    save_all(&accounts)
}

#[derive(Debug, Deserialize)]
struct ArtifactInfo {
    download_url: String,
    version: String,
}

/// Makes sure the authlib-injector agent is available locally.
pub async fn authlib_injector() -> crate::Result<PathBuf> {
    let dir = libraries_dir()?;
    let existing: Vec<PathBuf> = std::fs::read_dir(&dir)
        .map(|entries| {
            entries
                .flatten()
                .map(|entry| entry.path())
                .filter(|path| {
                    path.file_name()
                        .and_then(|name| name.to_str())
                        .is_some_and(|name| name.starts_with("authlib-injector"))
                })
                .collect()
        })
        .unwrap_or_default();
    if let Some(path) = existing.into_iter().next() {
        return Ok(path);
    }

    let info: ArtifactInfo = crate::util::fetch::INSECURE_REQWEST_CLIENT
        .get("https://authlib-injector.yushi.moe/artifact/latest.json")
        .send()
        .await?
        .error_for_status()?
        .json()
        .await
        .context("reading the authlib-injector release information")?;

    let bytes = crate::util::fetch::INSECURE_REQWEST_CLIENT
        .get(&info.download_url)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;

    let path = dir.join(format!("authlib-injector-{}.jar", info.version));
    std::fs::write(&path, bytes)?;
    Ok(path)
}

#[derive(Debug, Deserialize)]
struct YggdrasilProfile {
    id: String,
    name: String,
}

#[derive(Debug, Deserialize)]
struct YggdrasilAuthResponse {
    #[serde(rename = "accessToken")]
    access_token: String,
    #[serde(rename = "clientToken")]
    client_token: String,
    #[serde(rename = "selectedProfile")]
    selected_profile: Option<YggdrasilProfile>,
    #[serde(rename = "availableProfiles", default)]
    available_profiles: Vec<YggdrasilProfile>,
}

pub struct YggdrasilLogin {
    pub access_token: String,
    pub client_token: String,
    pub profile_id: Uuid,
    pub profile_name: String,
}

fn parse_simple_uuid(value: &str) -> crate::Result<Uuid> {
    if let Ok(uuid) = Uuid::parse_str(value) {
        return Ok(uuid);
    }
    let trimmed = value.trim();
    if trimmed.len() == 32 {
        let dashed = format!(
            "{}-{}-{}-{}-{}",
            &trimmed[0..8],
            &trimmed[8..12],
            &trimmed[12..16],
            &trimmed[16..20],
            &trimmed[20..32]
        );
        if let Ok(uuid) = Uuid::parse_str(&dashed) {
            return Ok(uuid);
        }
    }
    Err(anyhow!(
        "invalid profile id returned by the authentication server"
    ))
}

/// Whether an API root belongs to LittleSkin.
pub fn is_littleskin(api_root: &str) -> bool {
    api_root.to_lowercase().contains("littleskin")
}

/// Signs in against a Yggdrasil compatible server (LittleSkin included).
pub async fn yggdrasil_authenticate(
    api_root: &str,
    username: &str,
    password: &str,
) -> crate::Result<YggdrasilLogin> {
    let root = api_root.trim_end_matches('/');
    tracing::info!("yggdrasil: signing in to {root} as {username}");
    let client_token = Uuid::new_v4().simple().to_string();
    let body = serde_json::json!({
        "username": username,
        "password": password,
        "clientToken": client_token,
        "agent": { "name": "Minecraft", "version": 1 },
        "requestUser": false,
    });

    let response = crate::util::fetch::INSECURE_REQWEST_CLIENT
        .post(format!("{root}/authserver/authenticate"))
        .json(&body)
        .send()
        .await?;

    let status = response.status();
    if !status.is_success() {
        let text = response.text().await.unwrap_or_default();
        let message = serde_json::from_str::<serde_json::Value>(&text)
            .ok()
            .and_then(|value| {
                value
                    .get("errorMessage")
                    .and_then(|it| it.as_str())
                    .map(str::to_string)
            })
            .unwrap_or(text);
        tracing::warn!("yggdrasil: sign-in rejected ({status}): {message}");
        return Err(anyhow!("authentication failed ({status}): {message}"));
    }

    let auth: YggdrasilAuthResponse = response.json().await?;
    let profile = auth
        .selected_profile
        .or_else(|| auth.available_profiles.into_iter().next())
        .ok_or_else(|| anyhow!("the account has no Minecraft profile yet"))?;

    Ok(YggdrasilLogin {
        access_token: auth.access_token,
        client_token: auth.client_token,
        profile_id: parse_simple_uuid(&profile.id)?,
        profile_name: profile.name,
    })
}

/// Fetches the API metadata so it can be prefetched into the JVM arguments.
pub async fn yggdrasil_metadata(api_root: &str) -> crate::Result<String> {
    let root = api_root.trim_end_matches('/');
    let raw = crate::util::fetch::INSECURE_REQWEST_CLIENT
        .get(root)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;
    Ok(base64::prelude::BASE64_STANDARD.encode(raw))
}

/// Creates a third-party account and stores it alongside the Microsoft ones.
pub async fn add_yggdrasil_account(
    api_root: &str,
    username: &str,
    password: &str,
    kind: AccountKind,
) -> crate::Result<Credentials> {
    let state = State::get().await?;
    let login = match yggdrasil_authenticate(api_root, username, password).await {
        Ok(login) => login,
        Err(error) => {
            tracing::warn!("yggdrasil: authentication failed: {error:#}");
            return Err(error);
        }
    };
    tracing::info!(
        "yggdrasil: authenticated {} ({})",
        login.profile_name,
        login.profile_id
    );

    let credentials = Credentials {
        offline_profile: MinecraftProfile {
            id: login.profile_id,
            name: login.profile_name,
            skins: Vec::new(),
            capes: Vec::new(),
            fetch_time: None,
        },
        access_token: login.access_token,
        refresh_token: login.client_token,
        expires: Utc::now() + Duration::days(1),
        active: true,
    };

    credentials.upsert(&state.pool).await?;

    // A third-party login that points at LittleSkin is a LittleSkin account,
    // no matter which form was used to add it.
    let kind = if is_littleskin(api_root) {
        AccountKind::LittleSkin
    } else {
        kind
    };

    upsert(AccountMeta {
        uuid: login.profile_id.to_string(),
        kind,
        api_root: Some(api_root.trim_end_matches('/').to_string()),
        usage: 0,
        last_used: None,
    })?;

    Ok(credentials)
}

/// Extra JVM arguments needed to make a third-party account work in game.
pub async fn launch_arguments(uuid: Uuid) -> crate::Result<Vec<String>> {
    let Some(meta) = get(uuid) else {
        return Ok(Vec::new());
    };
    let Some(api_root) = meta.api_root else {
        return Ok(Vec::new());
    };

    let jar = authlib_injector().await?;
    let prefetched = yggdrasil_metadata(&api_root).await.unwrap_or_default();

    let mut arguments = vec![format!(
        "-javaagent:{}={}",
        jar.to_string_lossy(),
        api_root
    )];
    if !prefetched.is_empty() {
        arguments.push(format!(
            "-Dauthlibinjector.yggdrasil.prefetched={prefetched}"
        ));
    }
    Ok(arguments)
}
