mod config;
mod db;
mod error;
mod handler;
mod request;
mod response;
use axum::{
    routing::{get, post},
    serve, Router,
};
use config::Config;
use db::migration::run_migrations;
use deadpool_diesel::postgres::Pool;
use get_pass::get_password;
use handler::{label, shipment};
use reqwest::{header, Client, ClientBuilder};
use tracing::info;
#[derive(Clone)]
struct AppState {
    config: Config,
    pool: Pool,
    client: Client,
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let config: Config = confy::load_path("/etc/mondialrelay-api/config.toml")?;
    let mut db_uri = config.db_uri.clone();
    db_uri
        .set_password(Some(
            &get_password(&config.db_pass_path).expect("Invalid utf-8"),
        ))
        .unwrap();
    let pool = Pool::builder(deadpool_diesel::Manager::new(
        db_uri.as_str(),
        deadpool_diesel::Runtime::Tokio1,
    ))
    .build()?;
    run_migrations(&pool).await?;
    let mut headers = header::HeaderMap::new();
    headers.insert(
        header::ACCEPT,
        "application/xml"
            .parse()
            .expect("header value should be correct"),
    );
    headers.insert(
        header::CONTENT_TYPE,
        "text/value"
            .parse()
            .expect("header value should be correct"),
    );
    let client = ClientBuilder::new()
        .default_headers(headers)
        .build()
        .expect("value given to builder should be valid");
    let state = AppState {
        config,
        pool,
        client,
    };
    let listener =
        tokio::net::TcpListener::bind(format!("127.0.0.1:{}", state.config.listen_port)).await?;
    info!("Listening on port {}", state.config.listen_port);
    serve(listener, router(state)).await?;
    Ok(())
}

fn router(state: AppState) -> Router {
    Router::new()
        // all endpoint must be protected by authorization gateway allowing workers but not customers.
        .route("/shipment", post(shipment))
        // returns only the url, not the full pdf. client work must then fetch the url to get the pdf.
        .route("/label/:id_order", get(label))
        .with_state(state)
}
