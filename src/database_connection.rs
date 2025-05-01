use anyhow::Result;
use log::debug;
use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;
use std::sync::{Mutex, OnceLock};

static DATABASE_NAME: OnceLock<Mutex<String>> = OnceLock::new();

/// Represents the database connection configuration data
/// Contains credentials and connection details for both MySQL and Filemaker databases
#[derive(Serialize, Deserialize, Clone, Default)]
pub struct DatabaseConnectionData {
    /// MySQL host address
    pub host: String,
    /// MySQL username
    pub user: String,
    /// MySQL password
    pub password: String,
    /// Filemaker database credentials
    pub filemaker: FilemakerCredentials,
    /// Authentication hash
    pub hash: String,
}

/// Stores Filemaker database authentication credentials
#[derive(Serialize, Deserialize, Clone, Default)]
pub struct FilemakerCredentials {
    /// Filemaker username
    pub username: String,
    /// Filemaker password
    pub password: String,
}

impl DatabaseConnectionData {
    /// Retrieves database connection configuration from a remote JSON endpoint
    ///
    /// # Returns
    /// * `Result<Self>` - Database connection configuration if successful, error otherwise
    ///
    /// # Errors
    /// * When HTTP request fails
    /// * When JSON parsing fails
    pub async fn get() -> Result<Self> {
        if cfg!(debug_assertions) {
            let manifest_dir =
                std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
            let config_path = std::path::Path::new(&manifest_dir).join("dev-server.json");
            if !config_path.exists() {
                let default_config: DatabaseConnectionData = DatabaseConnectionData::default();
                let json = serde_json::to_string(&default_config);
                if let Ok(json) = json {
                    std::fs::write("dev-server.json", json)?;
                }
                return Err(anyhow::anyhow!("Configuration file not found"));
            }
            let config_json = std::fs::read_to_string(config_path)?;
            let dev_config: DatabaseConnectionData = serde_json::from_str(&config_json)?;
            Ok(dev_config)
        } else {
            use reqwest::Client;
            // Remote configuration endpoint
            let url = "https://lib.mardens.com/config.json";
            let client = Client::builder()
                .danger_accept_invalid_certs(true)
                .build()?;
            let response = client.get(url).send().await?;
            let credentials = response.json::<DatabaseConnectionData>().await?;
            Ok(credentials)
        }
    }
}

/// Creates a MySQL connection pool using the provided configuration
///
/// # Arguments
/// * `data` - Database connection configuration
///
/// # Returns
/// * `Result<MySqlPool>` - MySQL connection pool if successful, error otherwise
///
/// # Errors
/// * When connection to MySQL fails
pub async fn create_pool(data: &DatabaseConnectionData) -> Result<MySqlPool> {
    debug!("Creating MySQL production connection");
    let db = get_database_name()?;
    // Construct MySQL connection string and establish connection
    let pool = MySqlPool::connect(&format!(
        "mysql://{}:{}@{}/{}",
        data.user, data.password, data.host, db
    ))
    .await?;
    Ok(pool)
}

/// Sets the global database name to the provided string.
///
/// This function initializes a shared, thread-safe global variable (`DATABASE_NAME`)
/// with the name of the database.
/// If the database name has already been set and the method is called again,
/// it will return an error.
///
/// # Arguments
///
/// * `db` - A string slice representing the new database name to be set globally.
///
/// # Returns
///
/// * `Ok(())` - If the database name is successfully set.
/// * `Err(anyhow::Error)` - If there is an attempt to set the database name more than once,
///   or if the operation fails for any other reason.
///
/// # Example
///
/// ```norust
/// set_database_name("my_database").expect("Failed to set database name");
/// ```
pub fn set_database_name(db: &str) -> Result<()> {
    DATABASE_NAME
        .set(Mutex::new(db.to_string()))
        .map_err(|_| anyhow::anyhow!("Failed to set database name"))?;
    Ok(())
}
/// Retrieves the name of the database.
///
/// This function attempts to retrieve the global database name stored in `DATABASE_NAME`.
/// It ensures that the name has been initialized and can be accessed safely.
/// The name is guarded by a lock to handle potential concurrent access.
///
/// # Returns
///
/// * `Ok(String)` - The name of the database as a string if it is successfully retrieved.
/// * `Err(anyhow::Error)` - If the database name is not set, or if there is an error
///   acquiring the lock.
///
/// # Errors
///
/// - Returns an error if:
///   1. The `DATABASE_NAME` has not been initialized and is `None`.
///   2. The mutex guarding the database name fails to acquire the lock.
///
/// # Example
///
/// ```norust
/// match get_database_name() {
///     Ok(name) => println!("Database name: {}", name),
///     Err(e) => eprintln!("Error retrieving database name: {}", e),
/// }
/// ```
pub fn get_database_name() -> Result<String> {
    let name = DATABASE_NAME
        .get()
        .ok_or_else(|| anyhow::anyhow!("Database name not set"))?;
    let guard = name
        .lock()
        .map_err(|e| anyhow::anyhow!("Failed to acquire lock: {}", e))?;
    Ok(guard.to_string())
}
