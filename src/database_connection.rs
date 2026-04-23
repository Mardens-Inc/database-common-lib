use anyhow::{Result, anyhow};
use log::{debug, warn};
use serde::{Deserialize, Serialize};
use sqlx::mysql::{MySqlConnectOptions, MySqlPoolOptions};
use sqlx::{ConnectOptions, MySqlPool};
use std::sync::OnceLock;

/// Global database name, set once at process start via [`set_database_name`].
/// `OnceLock<String>` is sufficient here — the value is written once and only
/// read afterwards, so no additional locking is required.
static DATABASE_NAME: OnceLock<String> = OnceLock::new();

/// Remote URL used to fetch production configuration when no local env vars
/// are provided.
const REMOTE_CONFIG_URL: &str = "https://lib.mardens.com/config.json";

// ---------------------------------------------------------------------------
// Env var names (all optional). See `DatabaseConnectionData::get` for precedence.
// ---------------------------------------------------------------------------
const ENV_HOST: &str = "DB_HOST";
const ENV_USER: &str = "DB_USER";
const ENV_PASSWORD: &str = "DB_PASSWORD";
const ENV_PORT: &str = "DB_PORT";
const ENV_HASH: &str = "DB_HASH";
const ENV_FM_USER: &str = "DB_FILEMAKER_USER";
const ENV_FM_PASSWORD: &str = "DB_FILEMAKER_PASSWORD";
const ENV_DB_NAME: &str = "DB_NAME";

/// Database connection configuration.
///
/// Contains credentials and connection details for both MySQL and the
/// Filemaker integration.
#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct DatabaseConnectionData {
    /// MySQL host address.
    pub host: String,
    /// MySQL username.
    pub user: String,
    /// MySQL password.
    pub password: String,
    /// Filemaker database credentials.
    pub filemaker: FilemakerCredentials,
    /// Authentication hash.
    pub hash: String,
    /// MySQL server port. `None` lets sqlx use the MySQL default (3306).
    pub port: Option<u16>,
}

/// Filemaker database authentication credentials.
#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct FilemakerCredentials {
    /// Filemaker username.
    pub username: String,
    /// Filemaker password.
    pub password: String,
}

impl DatabaseConnectionData {
    /// Loads the database connection configuration.
    ///
    /// Resolution order:
    /// 1. **Base config**
    ///    - Debug builds: [`DatabaseConnectionData::default`] (empty fields).
    ///    - Release builds: fetched from [`REMOTE_CONFIG_URL`].
    /// 2. **Environment overrides** — any of the following variables that are
    ///    set will replace the corresponding field on the base config:
    ///    `DB_HOST`, `DB_USER`, `DB_PASSWORD`, `DB_PORT`, `DB_HASH`,
    ///    `DB_FILEMAKER_USER`, `DB_FILEMAKER_PASSWORD`.
    /// 3. **Validation** — in debug builds, `host`, `user`, and `password`
    ///    must be non-empty after env-var overlay; otherwise an error naming
    ///    the missing variables is returned.
    ///
    /// # Errors
    /// * Remote fetch fails (release only).
    /// * JSON parsing of the remote response fails (release only).
    /// * `DB_PORT` is set but cannot be parsed as a `u16`.
    /// * Required credentials are missing in debug builds.
    pub async fn get() -> Result<Self> {
        let mut config = if cfg!(debug_assertions) {
            DatabaseConnectionData::default()
        } else {
            fetch_remote_config().await?
        };

        apply_env_overrides(&mut config)?;

        if cfg!(debug_assertions) {
            validate_debug_config(&config)?;
        }

        Ok(config)
    }

    /// Returns a MySQL connection pool built from this configuration.
    ///
    /// Convenience wrapper around [`create_pool`].
    pub async fn get_pool(&self) -> Result<MySqlPool> {
        create_pool(self).await
    }
}

/// Fetches the remote production configuration JSON.
async fn fetch_remote_config() -> Result<DatabaseConnectionData> {
    use reqwest::Client;
    let client = Client::builder().danger_accept_invalid_certs(true).build()?;
    let response = client.get(REMOTE_CONFIG_URL).send().await?;
    let credentials = response.json::<DatabaseConnectionData>().await?;
    Ok(credentials)
}

/// Overlays any set environment variables onto `config`.
///
/// Only fields whose env var is present are modified. Returns an error if
/// `DB_PORT` is set but not a valid `u16`.
fn apply_env_overrides(config: &mut DatabaseConnectionData) -> Result<()> {
    if let Ok(v) = std::env::var(ENV_HOST) {
        config.host = v;
    }
    if let Ok(v) = std::env::var(ENV_USER) {
        config.user = v;
    }
    if let Ok(v) = std::env::var(ENV_PASSWORD) {
        config.password = v;
    }
    if let Ok(v) = std::env::var(ENV_HASH) {
        config.hash = v;
    }
    if let Ok(v) = std::env::var(ENV_FM_USER) {
        config.filemaker.username = v;
    }
    if let Ok(v) = std::env::var(ENV_FM_PASSWORD) {
        config.filemaker.password = v;
    }
    if let Ok(v) = std::env::var(ENV_PORT) {
        let port: u16 = v
            .parse()
            .map_err(|e| anyhow!("{ENV_PORT} must be a valid u16 (got {v:?}): {e}"))?;
        config.port = Some(port);
    }
    Ok(())
}

/// Ensures required MySQL credentials are present in debug builds.
fn validate_debug_config(config: &DatabaseConnectionData) -> Result<()> {
    let mut missing = Vec::new();
    if config.host.is_empty() {
        missing.push(ENV_HOST);
    }
    if config.user.is_empty() {
        missing.push(ENV_USER);
    }
    if config.password.is_empty() {
        missing.push(ENV_PASSWORD);
    }
    if !missing.is_empty() {
        return Err(anyhow!(
            "Missing required database env var(s) in debug build: {}. \
             Set them before running (e.g. DB_HOST, DB_USER, DB_PASSWORD).",
            missing.join(", ")
        ));
    }
    Ok(())
}

/// Creates a MySQL connection pool from `data`.
///
/// The global database name (set via [`set_database_name`]) is used as the
/// target database. Port defaults to MySQL's 3306 when `data.port` is `None`.
///
/// # Errors
/// * The database name has not been set.
/// * Connection to MySQL fails.
pub async fn create_pool(data: &DatabaseConnectionData) -> Result<MySqlPool> {
    debug!("Creating MySQL connection pool");
    let db = get_database_name()?;
    let mut options = MySqlConnectOptions::new()
        .log_statements(log::LevelFilter::Trace)
        .host(&data.host)
        .username(&data.user)
        .password(&data.password)
        .database(&db);

    if let Some(port) = data.port {
        options = options.port(port);
    }

    let pool = MySqlPoolOptions::new().connect_with(options).await?;
    Ok(pool)
}

/// Sets the global database name.
///
/// Must be called once before [`create_pool`] or [`DatabaseConnectionData::get_pool`].
/// Subsequent calls return an error — the name cannot be changed after it is set.
pub fn set_database_name(db: &str) -> Result<()> {
    DATABASE_NAME
        .set(db.to_string())
        .map_err(|_| anyhow!("Database name has already been set"))
}

/// Sets the global database name from the `DB_NAME` environment variable.
///
/// Returns an error if `DB_NAME` is unset or if the name has already been set.
pub fn set_database_name_from_env() -> Result<()> {
    let name = std::env::var(ENV_DB_NAME)
        .map_err(|_| anyhow!("{ENV_DB_NAME} is not set"))?;
    set_database_name(&name)
}

/// Returns the global database name previously set via [`set_database_name`].
///
/// # Errors
/// Returns an error if the database name has not yet been set.
pub fn get_database_name() -> Result<String> {
    DATABASE_NAME
        .get()
        .cloned()
        .ok_or_else(|| {
            warn!("Database name requested before being set");
            anyhow!("Database name has not been set; call set_database_name first")
        })
}
