use axum::serve;
use mondialrelay_api_lib::{router, AppState};
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let state = AppState::new(confy::load_path("/etc/mondialrelay-api/config.toml")?).await?;
    let listener =
        tokio::net::TcpListener::bind(format!("127.0.0.1:{}", state.config.listen_port)).await?;
    info!("Listening on port {}", state.config.listen_port);
    serve(listener, router(state)).await?;
    Ok(())
}
