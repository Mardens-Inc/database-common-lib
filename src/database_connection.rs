use anyhow::Result;
use log::debug;
use serde::Deserialize;
use sqlx::MySqlPool;

/// Represents the database connection configuration data
/// Contains credentials and connection details for both MySQL and Filemaker databases
#[derive(Deserialize, Clone)]
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
#[derive(Deserialize, Clone)]
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
    // Construct MySQL connection string and establish connection
    let pool = MySqlPool::connect(&format!(
        "mysql://{}:{}@{}/pricing",
        data.user, data.password, data.host
    ))
        .await?;
    Ok(pool)
}