/* src/main.rs */

use dotenvy::dotenv;
use fancy_log::{LogLevel, log, set_log_level};
use lazy_motd::lazy_motd;
use std::{env, net::SocketAddr, sync::Arc};
use tokio::sync::mpsc;

mod api;
mod config;
mod error;
mod manager;

use api::create_router;
use config::watch_config;
use manager::PathmapManager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // --- Initialization ---
    dotenv().ok();
    let level = env::var("LOG_LEVEL")
        .unwrap_or_else(|_| "info".to_string())
        .to_lowercase();
    let log_level = match level.as_str() {
        "debug" => LogLevel::Debug,
        "warn" => LogLevel::Warn,
        "error" => LogLevel::Error,
        _ => LogLevel::Info,
    };
    set_log_level(log_level);
    lazy_motd!();

    // --- Application Setup ---
    let manager = Arc::new(PathmapManager::new());
    let (tx, rx) = mpsc::channel(10);

    // Initial configuration load
    if let Err(e) = config::load_and_apply_config(manager.clone()).await {
        log(
            LogLevel::Error,
            &format!("Initial config load failed: {}", e),
        );
    }

    // Spawn the config watcher task
    let watcher_manager = manager.clone();
    tokio::spawn(async move {
        watch_config(tx, watcher_manager).await;
    });

    // Spawn the task to handle config updates
    let update_manager = manager.clone();
    tokio::spawn(async move {
        manager::handle_config_updates(rx, update_manager).await;
    });

    // --- Start Web Server ---
    let app = create_router(manager);
    let port = env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()?;
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    log(LogLevel::Info, &format!("Server listening on {}", addr));

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
