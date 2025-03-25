# Database Common Library

A Rust library providing common utilities for database connections and web server functionality. This library simplifies the setup of MySQL database connections and includes utilities for handling HTTP requests and static file serving.

## Features

### Database Connection Management
- Secure database configuration loading from remote JSON endpoints
- MySQL connection pool creation and management
- Support for both MySQL and Filemaker database credentials
- Automatic handling of SSL certificate validation

### Web Server Utilities
- Built-in static file serving capabilities
- Actix-web integration with route configuration
- HTTP server creation and configuration
- Built-in support for serving web assets from embedded directories

### Error Handling
- Comprehensive error handling system
- Custom error types for different scenarios
- Integration with actix-web's error handling
- Support for various error conversions (SQL, HTTP, IO errors)

## Usage

### Database Connection Setup

```rust
use database_common_lib::database_connection::{DatabaseConnectionData, create_pool};

async fn setup_database() -> anyhow::Result<()> {
  // Fetch database configuration
  let config = DatabaseConnectionData::get().await?;

  // Create MySQL connection pool
  let pool = create_pool(&config).await?;

  Ok(())
}
```

### Web Server Configuration

```rust
use database_common_lib::actix_extension::create_http_server;
use include_dir::include_dir;

async fn start_server() -> anyhow::Result<()> {
  // Create HTTP server with routes and static files
  let server = create_http_server(
    || Box::new(|cfg| {
      cfg.service(
        web::scope("/api")
                .route("/hello", web::get().to(hello))
      )
    }),
    include_dir!("target/wwwroot"),
    8080
  )?;

  // Start the server
  server.await?;

  Ok(())
}
```

### Complete Server with Database Integration

```rust
use actix_web::{web, HttpResponse, Responder};
use database_common_lib::{
    actix_extension::create_http_server,
    database_connection::{DatabaseConnectionData, create_pool},
};
use include_dir::include_dir;
use sqlx::MySqlPool;

// Example handler that uses the database
async fn get_users(db_pool: web::Data<MySqlPool>) -> impl Responder {
    match sqlx::query!("SELECT id, name FROM users LIMIT 10")
        .fetch_all(db_pool.get_ref())
        .await
    {
        Ok(users) => {
            let user_list = users
                .into_iter()
                .map(|user| serde_json::json!({ "id": user.id, "name": user.name }))
                .collect::<Vec<_>>();
            HttpResponse::Ok().json(user_list)
        }
        Err(_) => HttpResponse::InternalServerError().json(
            serde_json::json!({"error": "Failed to fetch users"})
        ),
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load database configuration
    let db_config = DatabaseConnectionData::get().await?;
    
    // Create connection pool
    let pool = create_pool(&db_config).await?;
    let db_data = web::Data::new(pool);
    
    // Create server with database integration
    let server = create_http_server(
        move || {
            let db_data = db_data.clone();
            Box::new(move |cfg| {
                cfg.app_data(db_data.clone())
                   .service(
                       web::scope("/api")
                           .route("/users", web::get().to(get_users))
                           .route("/health", web::get().to(health_check))
                   )
            })
        },
        include_dir!("target/wwwroot"),
        8080,
    )?;
    
    server.await?;
    Ok(())
}
```

### Error Handling

```rust
use database_common_lib::http_error::{Error, Result};

async fn my_handler() -> Result<String> {
    // Your logic here
    // The Result type automatically handles conversion of various error types
    Ok("Success".to_string())
}
```

## DatabaseConnectionData Structure

The `DatabaseConnectionData` struct provides a convenient way to manage and access database credentials:

```rust
#[derive(Deserialize, Clone)]
pub struct DatabaseConnectionData {
    // MySQL host address
    pub host: String,
    // MySQL username
    pub user: String,
    // MySQL password
    pub password: String,
    // Filemaker database credentials
    pub filemaker: FilemakerCredentials,
    // Authentication hash
    pub hash: String,
}
```

It includes a method to fetch configuration from a remote endpoint:

```rust
// Fetch database config automatically from a remote source
let config = DatabaseConnectionData::get().await?;
```

## Features At a Glance

- ✅ Remote configuration loading
- ✅ MySQL connection pooling
- ✅ Filemaker database support
- ✅ Static file serving
- ✅ Comprehensive error handling
- ✅ Actix-web integration
- ✅ SSL certificate handling

## License

This library is licensed under the GNU General Public License v3.0 (GPLv3). This means:

- You are free to:
  - Use this software for any purpose
  - Change the software to suit your needs
  - Share the software with others
  - Share the changes you make

- Under the following terms:
  - You must disclose source code of your modifications
  - You must license your modifications under the same GPLv3 license
  - You must state significant changes made to the software
  - You must preserve copyright and license notices

For more details, see the [LICENSE](LICENSE) file in the project repository or visit [GNU GPL v3.0](https://www.gnu.org/licenses/gpl-3.0.en.html).