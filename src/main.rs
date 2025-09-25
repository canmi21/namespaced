/* src/main.rs */

use dotenvy::dotenv;
use fancy_log::{LogLevel, log, set_log_level};
use lazy_motd::lazy_motd;
use std::{env, net::SocketAddr, sync::Arc};
use tokio::sync::{Mutex, mpsc};

mod admin;
mod api;
mod config;
mod error;
mod manager;

use api::create_router;
use config::watch_config;
use manager::PathmapManager;

// The shared state for our application, accessible by all handlers.
pub struct AppState {
    pub manager: Arc<PathmapManager>,
    pub config_lock: Arc<Mutex<()>>, // Used to prevent race conditions on config file writes
}

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
    let app_state = Arc::new(AppState {
        manager: Arc::new(PathmapManager::new()),
        config_lock: Arc::new(Mutex::new(())),
    });

    let (tx, rx) = mpsc::channel(10);

    // Initial configuration load
    if let Err(e) = config::load_and_apply_config(app_state.manager.clone()).await {
        log(
            LogLevel::Error,
            &format!("Initial config load failed: {}", e),
        );
    }

    // Spawn the config watcher task
    tokio::spawn(watch_config(tx, app_state.manager.clone()));

    // Spawn the task to handle config updates
    tokio::spawn(manager::handle_config_updates(
        rx,
        app_state.manager.clone(),
    ));

    // --- Start Web Server ---
    let app = create_router(app_state);
    let port = env::var("PORT")
        .unwrap_or_else(|_| "19950".to_string())
        .parse::<u16>()?;
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    log(LogLevel::Info, &format!("Server listening on {}", addr));

    let listener = tokio::net::TcpListener::bind(addr).await?;

    // --- Run with Graceful Shutdown ---
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

// Listens for the shutdown signal (Ctrl+C or SIGTERM).
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    log(
        LogLevel::Info,
        "Signal received, starting graceful shutdown.",
    );
}
