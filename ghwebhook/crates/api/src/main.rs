use rocket::http::Method;
use rocket_cors::{AllowedHeaders, AllowedOrigins, CorsOptions};

use crate::services::create_rabbitmq_producer;

#[macro_use]
extern crate rocket;

mod config;
mod routes;
mod services;
mod types;

fn make_cors() -> rocket_cors::Cors {
    CorsOptions {
        allowed_origins: AllowedOrigins::all(),
        allowed_methods: vec![
            Method::Get,
            Method::Post,
            Method::Options,
            Method::Delete,
            Method::Put,
        ]
        .into_iter()
        .map(From::from)
        .collect(),
        allowed_headers: AllowedHeaders::all(),
        allow_credentials: true,
        max_age: Some(3600),
        ..Default::default()
    }
    .to_cors()
    .expect("Failed to create CORS")
}

#[tokio::main]
async fn main() -> Result<(), lib::errors::AppError> {
    let cors = make_cors();

    let config = config::AppConfig::new()?;

    let rabbitmq_producer = create_rabbitmq_producer(&config, "ghwebhook", 5).await?;

    let state = types::AppState { rabbitmq_producer };

    let rocket = rocket::build()
        .attach(cors)
        .mount("/", routes![routes::webhook]);

    rocket.manage(state).launch().await?;

    Ok(())
}
