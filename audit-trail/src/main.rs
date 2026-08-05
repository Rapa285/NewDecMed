mod handlers;
mod constants;
mod types;
mod audit_error;
mod macros;

use std::{env, error::Error, str::FromStr, sync::Arc};
use axum::{
    middleware,
    routing::{get, post, put},
    Router,
};
use tokio::net::TcpListener;
use handlers::Handlers;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    
    let app = Router::new()
        .route("/", get(|| async { "Service Audit Trail System siap!" }))
        .route("/new-event", post(Handlers::add_and_pin_to_ipfs));

    let port = env::var("PORT")?;

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap();
    
    println!("Service berjalan dan mendengarkan di port {}...", port);

    axum::serve(listener, app).await.unwrap();

    Ok(())

}