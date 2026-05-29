//! Local credential persistence and resolution for the CLI.
//!
//! The publishable `koban` library stays environment-first and free of disk or
//! keychain I/O. This CLI-only module adds a stored credential layer on top:
//! `koban auth login` writes a token here, and normal commands resolve a token
//! through [`resolve`] before building a [`koban::Config`].
//!
//! Resolution precedence (highest first):
//! 1. `INVOICE_NINJA_API_TOKEN` / `INVOICE_NINJA_BASE_URL` environment variables
//! 2. the OS keychain (when the stored config marks the token as keychain-backed)
//! 3. the stored config file (`config.json`)
//!
//! Environment variables always win so agents and CI stay deterministic.

use std::{
    env, fs,
    path::{Path, PathBuf},
};

use directories::ProjectDirs;
use koban::{API_TOKEN_ENV, BASE_URL_ENV, DEFAULT_BASE_URL, KobanError, Result};
use serde::{Deserialize, Serialize};

/// Override for the config directory. Keeps tests hermetic and gives power
/// users an escape hatch from the platform default.
const CONFIG_DIR_ENV: &str = "KOBAN_CONFIG_DIR";
const CONFIG_FILE: &str = "config.json";

#[cfg(feature = "keychain")]
const KEYCHAIN_SERVICE: &str = "koban";
#[cfg(feature = "keychain")]
const KEYCHAIN_USER: &str = "api_token";

/// On-disk credential record. The token is stored inline (`api_token`) by
/// default, or held in the OS keychain when `keychain` is true.
#[derive(Debug, Default, Serialize, Deserialize)]
pub(crate) struct StoredConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_token: Option<String>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub keychain: bool,
}

/// Where a resolved token came from. Reported by `koban auth status`; never
/// carries the token itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TokenSource {
    Env,
    Keychain,
    File,
    None,
}

impl TokenSource {
    pub(crate) fn label(self) -> &'static str {
        match self {
            TokenSource::Env => "environment",
            TokenSource::Keychain => "keychain",
            TokenSource::File => "config file",
            TokenSource::None => "none",
        }
    }
}

/// Summary for `koban auth status`. Deliberately excludes the token value.
#[derive(Debug)]
pub(crate) struct AuthStatus {
    pub source: TokenSource,
    pub base_url: String,
    pub config_path: PathBuf,
}

fn credential_error(message: impl Into<String>) -> KobanError {
    KobanError::Credential {
        message: message.into(),
    }
}

/// Resolve the config directory, honoring the `KOBAN_CONFIG_DIR` override.
pub(crate) fn config_dir() -> Result<PathBuf> {
    if let Some(dir) = env::var_os(CONFIG_DIR_ENV) {
        let dir = PathBuf::from(dir);
        if dir.as_os_str().is_empty() {
            return Err(credential_error(format!(
                "{CONFIG_DIR_ENV} is set but empty"
            )));
        }
        return Ok(dir);
    }

    ProjectDirs::from("", "", "koban")
        .map(|dirs| dirs.config_dir().to_path_buf())
        .ok_or_else(|| credential_error("could not determine a config directory for this platform"))
}

pub(crate) fn config_path() -> Result<PathBuf> {
    Ok(config_dir()?.join(CONFIG_FILE))
}

fn load_file() -> Result<StoredConfig> {
    let path = config_path()?;
    match fs::read_to_string(&path) {
        Ok(contents) => serde_json::from_str(&contents).map_err(|source| {
            credential_error(format!("could not parse {}: {source}", path.display()))
        }),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(StoredConfig::default()),
        Err(error) => Err(credential_error(format!(
            "could not read {}: {error}",
            path.display()
        ))),
    }
}

fn write_file(path: &Path, stored: &StoredConfig) -> Result<()> {
    let json = serde_json::to_string_pretty(stored)
        .map_err(|source| credential_error(format!("could not serialize config: {source}")))?;
    write_secure(path, &json)
}

/// Write `contents` to `path` with owner-only (0600) permissions. The mode is
/// applied at creation on unix and re-enforced afterward so a pre-existing file
/// is also tightened.
fn write_secure(path: &Path, contents: &str) -> Result<()> {
    #[cfg(unix)]
    {
        use std::io::Write;
        use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

        let mut file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(path)
            .map_err(|source| {
                credential_error(format!("could not write {}: {source}", path.display()))
            })?;
        file.write_all(contents.as_bytes()).map_err(|source| {
            credential_error(format!("could not write {}: {source}", path.display()))
        })?;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600)).map_err(|source| {
            credential_error(format!(
                "could not set permissions on {}: {source}",
                path.display()
            ))
        })?;
    }

    #[cfg(not(unix))]
    {
        fs::write(path, contents).map_err(|source| {
            credential_error(format!("could not write {}: {source}", path.display()))
        })?;
    }

    Ok(())
}

/// Persist a token (and optional base URL), choosing keychain or file storage.
/// Returns the config file path that was written.
pub(crate) fn save(base_url: Option<String>, token: &str, use_keychain: bool) -> Result<PathBuf> {
    let dir = config_dir()?;
    fs::create_dir_all(&dir).map_err(|source| {
        credential_error(format!("could not create {}: {source}", dir.display()))
    })?;
    let path = dir.join(CONFIG_FILE);

    let mut stored = load_file()?;
    if base_url.is_some() {
        stored.base_url = base_url;
    }

    if use_keychain {
        keychain_set(token)?;
        stored.keychain = true;
        stored.api_token = None;
    } else {
        // Drop any stale keychain secret so the file becomes the single source.
        let _ = keychain_delete();
        stored.keychain = false;
        stored.api_token = Some(token.to_string());
    }

    write_file(&path, &stored)?;
    Ok(path)
}

/// Resolve `(base_url, token)`.
///
/// Base-URL resolution is coupled to the token's source so a credential never
/// crosses sources: an environment token is paired only with an environment (or
/// default) base URL — never a stored one — matching the previous
/// `Config::from_env` behavior and keeping env-driven runs self-contained.
pub(crate) fn resolve() -> Result<(String, String)> {
    // Propagate read/parse errors: a corrupt config should surface a clear
    // error, not silently fall back to defaults and report a missing token.
    let stored = load_file()?;

    if let Some(token) = env_non_empty(API_TOKEN_ENV) {
        return Ok((resolve_base_url(TokenSource::Env, &stored), token));
    }

    let (source, token) = resolve_stored_token(&stored)?;
    Ok((resolve_base_url(source, &stored), token))
}

/// Resolve a token from stored credentials (keychain or file), reporting which.
fn resolve_stored_token(stored: &StoredConfig) -> Result<(TokenSource, String)> {
    if stored.keychain {
        return keychain_get()?
            .map(|token| (TokenSource::Keychain, token))
            .ok_or(KobanError::MissingToken);
    }

    stored
        .api_token
        .as_ref()
        .filter(|token| !token.trim().is_empty())
        .map(|token| (TokenSource::File, token.clone()))
        .ok_or(KobanError::MissingToken)
}

/// Resolve the base URL for a token of the given source. An explicit
/// `INVOICE_NINJA_BASE_URL` always wins; otherwise an environment token falls
/// back to the default (never a stored URL), while a stored token uses its
/// stored URL.
fn resolve_base_url(source: TokenSource, stored: &StoredConfig) -> String {
    if let Some(base_url) = env_non_empty(BASE_URL_ENV) {
        return base_url;
    }
    match source {
        TokenSource::Keychain | TokenSource::File => stored.base_url.clone(),
        TokenSource::Env | TokenSource::None => None,
    }
    .unwrap_or_else(|| DEFAULT_BASE_URL.to_string())
}

/// The base URL recorded in the config file, if any. Used by `login` to default
/// to (and re-verify against) the previously stored host.
pub(crate) fn stored_base_url() -> Result<Option<String>> {
    Ok(load_file()?.base_url)
}

/// Report the active credential source without exposing the token.
pub(crate) fn status() -> Result<AuthStatus> {
    let stored = load_file()?;

    let source = if env_non_empty(API_TOKEN_ENV).is_some() {
        TokenSource::Env
    } else if stored.keychain {
        // The config points at the keychain: surface a locked/unreachable
        // backend instead of masking it as "not authenticated".
        if keychain_get()?.is_some() {
            TokenSource::Keychain
        } else {
            TokenSource::None
        }
    } else if stored
        .api_token
        .as_ref()
        .is_some_and(|token| !token.trim().is_empty())
    {
        TokenSource::File
    } else {
        TokenSource::None
    };

    let base_url = resolve_base_url(source, &stored);

    Ok(AuthStatus {
        source,
        base_url,
        config_path: config_path()?,
    })
}

/// Remove any stored credential (file token + keychain entry). Returns whether
/// something was actually removed. A stored `base_url` is preserved.
pub(crate) fn clear() -> Result<bool> {
    // Surface keychain failures: if a secret exists but the backend is locked or
    // unreachable, logout must error rather than orphan the stored credential.
    let mut removed = keychain_delete()?;

    let path = config_path()?;
    if path.exists() {
        let mut stored = load_file()?;
        if stored.api_token.take().is_some() || stored.keychain {
            removed = true;
        }
        stored.keychain = false;

        if stored.base_url.is_some() {
            write_file(&path, &stored)?;
        } else {
            fs::remove_file(&path).map_err(|source| {
                credential_error(format!("could not remove {}: {source}", path.display()))
            })?;
        }
    }

    Ok(removed)
}

fn env_non_empty(key: &str) -> Option<String> {
    env::var(key).ok().filter(|value| !value.trim().is_empty())
}

// --- Keychain backend (feature-gated) ---------------------------------------

#[cfg(feature = "keychain")]
fn keychain_entry() -> Result<keyring::Entry> {
    keyring::Entry::new(KEYCHAIN_SERVICE, KEYCHAIN_USER)
        .map_err(|source| credential_error(format!("keychain: {source}")))
}

#[cfg(feature = "keychain")]
fn keychain_set(token: &str) -> Result<()> {
    keychain_entry()?
        .set_password(token)
        .map_err(|source| credential_error(format!("keychain: {source}")))
}

#[cfg(feature = "keychain")]
fn keychain_get() -> Result<Option<String>> {
    match keychain_entry()?.get_password() {
        Ok(token) => Ok(Some(token)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(source) => Err(credential_error(format!("keychain: {source}"))),
    }
}

#[cfg(feature = "keychain")]
fn keychain_delete() -> Result<bool> {
    match keychain_entry()?.delete_credential() {
        Ok(()) => Ok(true),
        Err(keyring::Error::NoEntry) => Ok(false),
        Err(source) => Err(credential_error(format!("keychain: {source}"))),
    }
}

#[cfg(not(feature = "keychain"))]
fn keychain_set(_token: &str) -> Result<()> {
    Err(credential_error(
        "this build has no keychain support; rebuild with the default `keychain` feature or store the token without --keychain",
    ))
}

#[cfg(not(feature = "keychain"))]
fn keychain_get() -> Result<Option<String>> {
    Err(credential_error(
        "this build has no keychain support; re-store the token without --keychain",
    ))
}

#[cfg(not(feature = "keychain"))]
fn keychain_delete() -> Result<bool> {
    Ok(false)
}
