use axum::{Router, routing::get};
use config::Config;
use db::migration::run_migrations;
use deadpool_diesel::postgres::Pool;
use get_pass::get_password;
use handler::{label, shipment};
use reqwest::{
    Client, ClientBuilder,
    header::{self, ACCEPT, CONTENT_TYPE},
};

pub mod config;
pub mod db;
pub mod error;
pub mod handler;
pub mod request;

#[derive(Clone)]
pub struct AppState {
    // Configuration that the program will run with.
    // roadmap could include allowing to use command line args and environments variable.
    pub config: Config,
    // Database pool connections
    pub pool: Pool,
    // reqwest client to interact with Mondial Relay API
    pub client: Client,
}

impl AppState {
    pub async fn new(config: Config) -> Result<Self, Box<dyn std::error::Error>> {
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
            ACCEPT,
            "application/xml"
                .parse()
                .expect("header value should be correct"),
        );
        headers.insert(
            CONTENT_TYPE,
            "text/xml".parse().expect("header value should be correct"),
        );
        let client = ClientBuilder::new()
            .default_headers(headers)
            .build()
            .expect("value given to builder should be valid");
        Ok(AppState {
            config,
            pool,
            client,
        })
    }
}
pub fn router(state: AppState) -> Router {
    Router::new()
        // all endpoint must be protected by authorization gateway allowing workers but not customers.
        .route("/shipment", axum::routing::post(shipment))
        // returns only the url, not the full pdf. client work must then fetch the url to get the pdf.
        .route("/label/:id_order", get(label))
        .with_state(state)
}
