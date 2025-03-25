use actix_web::{web, App, HttpResponse, Responder};
use anyhow::Result;
use database_common_lib::{
	actix_extension::create_http_server,
	database_connection::{DatabaseConnectionData, create_pool},
};
use include_dir::include_dir;
use std::sync::Arc;
use sqlx::MySqlPool;

// Handler that uses database connection
async fn get_users(db_pool: web::Data<MySqlPool>) -> impl Responder {
	// Example query using the database pool
	match sqlx::query!("SELECT id, name FROM users LIMIT 10")
		.fetch_all(db_pool.get_ref())
		.await
	{
		Ok(users) => {
			// Convert the users to a format that can be returned as JSON
			let user_list: Vec<_> = users
				.into_iter()
				.map(|user| serde_json::json!({ "id": user.id, "name": user.name }))
				.collect();

			HttpResponse::Ok().json(user_list)
		}
		Err(err) => {
			eprintln!("Database query error: {}", err);
			HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch users from database"
            }))
		}
	}
}

async fn health_check(db_pool: web::Data<MySqlPool>) -> impl Responder {
	// Test database connection is working
	match sqlx::query("SELECT 1").execute(db_pool.get_ref()).await {
		Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "status": "healthy",
            "database": "connected"
        })),
		Err(_) => HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "status": "unhealthy",
            "database": "disconnected"
        })),
	}
}

#[tokio::main]
async fn main() -> Result<()> {
	// Load database configuration from remote JSON endpoint
	let db_config = DatabaseConnectionData::get().await?;

	// Create MySQL connection pool
	let pool = create_pool(&db_config).await?;

	// Wrap the pool in web::Data for sharing across handlers
	let db_data = web::Data::new(pool);

	// Create the HTTP server
	let server = create_http_server(
		move || {
			let db_data = db_data.clone();

			Box::new(move |cfg| {
				cfg.app_data(db_data.clone())
				   .service(
					   web::scope("/api")
						   .route("/users", web::get().to(get_users))
						   .route("/health", web::get().to(health_check))
				   );
			})
		},
		include_dir!("target/wwwroot"),
		8080, // Port number
	)?;

	println!("Server running at http://localhost:8080");

	// Start the server
	server.await?;

	Ok(())
}