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
use database_common_lib::actix_extension::{AssetsAppConfig, create_http_server};

async fn start_server() -> anyhow::Result<()> {
    // Create HTTP server on port 8080
    let server = create_http_server(8080)?;
    
    // Start the server
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