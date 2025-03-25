use actix_web::error::HttpError;
use actix_web::http::header::ToStrError;
use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use anyhow::anyhow;

/// Custom error types for handling various error scenarios in the application
#[derive(thiserror::Error, Debug)]
pub enum Error {
    // Represents unspecified internal errors
    #[allow(dead_code)]
    #[error("an unspecified internal error occurred: * `lib.rs` sibling file text :*:
```rust
pub mod asset_endpoint;
pub mod data_database_connection;
pub mod http_error;

```")]
    InternalError(anyhow::Error),

    // Generic error type for miscellaneous errors
    #[allow(dead_code)]
    #[error(transparent)]
    Other(anyhow::Error),

    // General application error wrapper
    #[error("an error has occurred: {0:?}")]
    Anyhow(anyhow::Error),

    // Specific error for header parsing failures
    #[error("unable to parse headers: {0:?}")]
    HeaderParse(ToStrError),
}

/// Implementation of ResponseError trait for custom Error enum
impl ResponseError for Error {
    /// Determines the appropriate HTTP status code based on the error type
    fn status_code(&self) -> StatusCode {
        match &self {
            // Return 500 Internal Server Error for internal errors
            Self::InternalError(_) | Self::Other(_) => StatusCode::INTERNAL_SERVER_ERROR,
            // Return 400 Bad Request for all other error types
            _ => StatusCode::BAD_REQUEST,
        }
    }

    /// Converts the error into an HTTP response
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

/// Conversion from anyhow::Error to custom Error type
impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Self {
        Error::Anyhow(err)
    }
}

/// Conversion from ToStrError to custom Error type
impl From<ToStrError> for Error {
    fn from(err: ToStrError) -> Self {
        Error::HeaderParse(err)
    }
}

/// Conversion from std::io::Error to custom Error type
impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Anyhow(anyhow::Error::new(err))
    }
}

/// Conversion from sqlx::Error to custom Error type
impl From<sqlx::Error> for Error {
    fn from(err: sqlx::Error) -> Self {
        Error::Anyhow(anyhow::Error::new(err))
    }
}

/// Conversion from HttpError to custom Error type
impl From<HttpError> for Error {
    fn from(err: HttpError) -> Self {
        Error::Anyhow(anyhow::Error::new(err))
    }
}

/// Conversion from HttpResponse to custom Error type
impl From<HttpResponse> for Error {
    fn from(err: HttpResponse) -> Self {
        Error::Anyhow(anyhow!(
            "HTTP response error: {}",
            err.status().canonical_reason().unwrap_or("")
        ))
    }
}



// Type alias for Result using custom Error type
pub type Result<T> = std::result::Result<T, Error>;