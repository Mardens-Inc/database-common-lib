use actix_web::{web, App, HttpResponse, Responder};
use anyhow::Result;
use database_common_lib::actix_extension::create_http_server;
use include_dir::include_dir;

// Define some handler functions
async fn hello() -> impl Responder {
	HttpResponse::Ok().body("Hello, world!")
}

async fn echo(info: web::Path<String>) -> impl Responder {
	HttpResponse::Ok().body(format!("You said: {}", info))
}

async fn health_check() -> impl Responder {
	HttpResponse::Ok().json(serde_json::json!({"status": "ok"}))
}

#[tokio::main]
async fn main() -> Result<()> {
	// Create the HTTP server with configuration
	let server = create_http_server(
		move || {
			Box::new(move |cfg| {
				cfg.service(
					// Register a scope with multiple routes
					web::scope("/api")
						.route("/hello", web::get().to(hello))
						.route("/echo/{message}", web::get().to(echo))
						.route("/health", web::get().to(health_check))
				);
			})
		},
		include_dir!("target/wwwroot"),
		8080, // Port number
	)?;

	// Start the server
	println!("Server running at http://localhost:8080");
	server.await?;

	Ok(())
}