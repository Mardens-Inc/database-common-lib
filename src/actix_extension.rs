use actix_files::file_extension_to_mime;
use actix_web::dev::{Response, Server, ServiceFactory};
use actix_web::error::ErrorInternalServerError;
use actix_web::{
    get, middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder,
};
use anyhow::Result;
use include_dir::{include_dir, Dir};
use log::error;
use serde_json::json;
use vite_actix::ViteAppFactory;

// Static directory including all files under `target/wwwroot`.
// This static directory is used to embed files into the binary at compile time.
// The `WWWROOT` directory will be used to serve static files such as `index.html`.
static WWWROOT: Dir = include_dir!("target/wwwroot");

/// Serves the index.html file from the embedded static directory.
///
/// # Arguments
///
/// * `_req` - The HTTP request object (unused but required by the framework)
///
/// # Returns
///
/// * `Ok(impl Responder)` - HTTP response containing the index.html file
/// * `Err(Error)` - Internal server error if the file is not found
pub async fn index(_req: HttpRequest) -> anyhow::Result<impl Responder, Error> {
    if let Some(file) = WWWROOT.get_file("index.html") {
        let body = file.contents();
        return Ok(HttpResponse::Ok().content_type("text/html").body(body));
    }
    Err(ErrorInternalServerError("Failed to find index.html"))
}

/// Handles requests for static assets from the /assets directory.
///
/// # Arguments
///
/// * `file` - Path parameter containing the requested asset file name
///
/// # Returns
///
/// * `Ok(HttpResponse)` - Response containing the requested asset with appropriate MIME type
/// * `Err(Error)` - Internal server error if the file is not found
#[get("")]
async fn assets(file: web::Path<String>) -> impl Responder {
    // Attempt to retrieve the requested asset from the embedded directory
    if let Some(file) = WWWROOT.get_file(format!("assets/{}", file.as_str())) {
        let body = file.contents();
        return Ok(HttpResponse::Ok()
            .content_type(file_extension_to_mime(
                file.path().extension().unwrap().to_str().unwrap(),
            ))
            .body(body));
    }
    Err(ErrorInternalServerError(format!("Failed to find {}", file)))
}

/// Trait for configuring static asset routes in the application.
pub trait AssetsAppConfig {
    fn configure_routes(self) -> Self;
}

/// Implementation of AssetsAppConfig for the Actix-web App.
///
/// Configures routes differently based on debug/release mode:
/// - Release mode: Serves static files from embedded directory
/// - Debug mode: Uses Vite development server
impl<T> AssetsAppConfig for App<T>
where
    T: ServiceFactory<
            actix_web::dev::ServiceRequest,
            Config = (),
            Error = Error,
            InitError = (),
        >,
{
    fn configure_routes(self) -> Self {
        if !cfg!(debug_assertions) {
            // Production mode: serve static files from embedded directory
            self.default_service(web::route().to(index))
                .service(web::scope("/assets/{file:.*}").service(assets))
        } else {
            // Development mode: use Vite development server
            self.configure_vite()
        }
    }
}

pub fn create_http_server(port: u16) -> Result<Server> {
    let server = HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default()) // Add logger middleware
            .app_data(
                web::JsonConfig::default()
                    .limit(4096) // Set JSON payload size limit
                    .error_handler(|err, _req| {
                        error!("Failed to parse JSON: {}", err);
                        let error = json!({ "error": format!("{}", err) });
                        actix_web::error::InternalError::from_response(
                            err,
                            HttpResponse::BadRequest().json(error),
                        )
                        .into()
                    }),
            )
            .configure_routes()
    })
    .workers(4) // Set number of workers
    .bind(format!("0.0.0.0:{port}", port = port))? // Bind to specified port
    .run();
    Ok(server)
}
