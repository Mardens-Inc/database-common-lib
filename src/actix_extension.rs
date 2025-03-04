use actix_files::file_extension_to_mime;
use actix_web::dev::Server;
use actix_web::error::ErrorInternalServerError;
use actix_web::{get, middleware, Error, HttpRequest, HttpResponse, Responder};
use anyhow::Result;
use include_dir::{include_dir, Dir};
use log::error;
use serde_json::json;
use std::sync::Arc;
use vite_actix::ViteAppFactory;

use actix_web::web::Data;
use actix_web::{
    dev::{ServiceFactory, ServiceRequest}, web,
    App,
    HttpServer,
};

/// Serves the index.html file from the embedded static directory.
///
/// # Arguments
///
/// * `wwwroot` - Embedded static directory containing the files
/// * `_req` - The HTTP request object (unused but required by the framework)
///
/// # Returns
///
/// * `Ok(impl Responder)` - HTTP response containing the index.html file
/// * `Err(Error)` - Internal server error if the file is not found
pub async fn index(
    wwwroot: Data<Dir<'static>>,
    _req: HttpRequest,
) -> Result<impl Responder, Error> {
    if let Some(file) = wwwroot.get_file("index.html") {
        let body = file.contents();
        return Ok(HttpResponse::Ok().content_type("text/html").body(body));
    }
    Err(ErrorInternalServerError("Failed to find index.html"))
}

/// Handles requests for static assets from the /assets directory.
///
/// # Arguments
///
/// * `wwwroot` - Embedded static directory containing the files
/// * `file` - Path parameter containing the requested asset file name
///
/// # Returns
///
/// * `Ok(HttpResponse)` - Response containing the requested asset with appropriate MIME type
/// * `Err(Error)` - Internal server error if the file is not found
#[get("")]
async fn assets(wwwroot: Data<Dir<'static>>, file: web::Path<String>) -> impl Responder {
    if let Some(file) = wwwroot.get_file(format!("assets/{}", file.as_str())) {
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
    fn configure_routes(self, wwwroot: Data<Dir<'static>>) -> Self;
}

/// Implementation of AssetsAppConfig for the Actix-web App.
///
/// Configures routes differently based on debug/release mode:
/// - Release mode: Serves static files from embedded directory
/// - Debug mode: Uses Vite development server
impl<T> AssetsAppConfig for App<T>
where
    T: ServiceFactory<ServiceRequest, Config = (), Error = Error, InitError = ()>,
{
    fn configure_routes(self, wwwroot: Data<Dir<'static>>) -> Self {
        if !cfg!(debug_assertions) {
            self.app_data(wwwroot.clone())
                .default_service(web::route().to(index))
                .service(web::scope("/assets/{file:.*}").service(assets))
        } else {
            self.app_data(wwwroot.clone()).configure_vite()
        }
    }
}

/// Creates and configures an HTTP server with customized middleware and JSON handling
///
/// # Arguments
/// * `factory` - A function that configures web service routes and settings
/// * `wwwroot` - Embedded static directory containing the files
/// * `port` - The port number on which the server will listen
///
/// # Returns
/// * `Result<Server, std::io::Error>` - Configured server instance if successful, error otherwise
///
/// # Type Parameters
/// * `F` - Factory function type that implements required traits
/// * `T` - Return type of the factory function
pub fn create_http_server<F>(
    factory: F,
    wwwroot: Dir<'static>,
    port: u16,
) -> Result<Server, std::io::Error>
where
    F: Fn() -> Box<dyn FnOnce(&mut web::ServiceConfig) + Send + 'static> + Send + Clone + 'static,
{
    let wwwroot = Data::new(wwwroot);
    let server = HttpServer::new(move || {
        let config_fn = factory();
        App::new()
            .wrap(middleware::Logger::default())
            .app_data(
                web::JsonConfig::default()
                    .limit(4096)
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
            .app_data(wwwroot.clone())
            .configure_routes(wwwroot)
            .configure(|cfg| config_fn(cfg))
    })
    .workers(4)
    .bind(format!("0.0.0.0:{}", port))?
    .run();
    Ok(server)
}

#[test]
fn test_create_http_server() {
    let wwwroot: Dir = include_dir!("target/wwwroot");
    // Create the HTTP server with the factory closure
    let server_result = create_http_server(
        || {
            Box::new(|cfg: &mut web::ServiceConfig| {
                cfg.service(web::scope("/api").route(
                    "/hello",
                    web::get().to(|| async { HttpResponse::Ok().body("Hello, world!") }),
                ));
            })
        },
        wwwroot,
        8080,
    );

    assert!(server_result.is_ok());
}
