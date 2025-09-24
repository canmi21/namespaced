/* src/config.rs */

use crate::{error::AppError, manager::PathmapManager};
use fancy_log::{LogLevel, log};
use notify::{RecursiveMode, Watcher, event::EventKind};
use serde::Deserialize;
use std::{collections::HashMap, path::Path, sync::Arc};
use tokio::fs;
use tokio::sync::mpsc::{self, Sender}; // Corrected: Import mpsc and Sender correctly

pub const CONFIG_PATH: &str = "/opt/namespaced/pathmap.json";

// Represents the structure of the JSON configuration file.
#[derive(Deserialize, Debug, Clone)]
pub struct NamespacedConfig {
    #[serde(flatten)]
    pub projects: HashMap<String, String>, // project_name -> base_path
}

// Loads the configuration file from disk.
pub async fn load_config() -> Result<NamespacedConfig, AppError> {
    let path = Path::new(CONFIG_PATH);

    // Create default config if it doesn't exist
    if !path.exists() {
        log(
            LogLevel::Warn,
            &format!("Config not found at {}. Creating default.", CONFIG_PATH),
        );
        let parent = path
            .parent()
            .ok_or_else(|| AppError::ConfigError("Invalid config path".to_string()))?;
        fs::create_dir_all(parent).await?;
        let default_config = r#"{
    "example_project": "/opt/ns/example"
}"#;
        fs::write(path, default_config).await?;
    }

    let content = fs::read_to_string(path).await?;
    let config: NamespacedConfig = serde_json::from_str(&content)?;
    Ok(config)
}

// Applies the loaded configuration to the PathmapManager.
pub async fn apply_config(config: NamespacedConfig, manager: Arc<PathmapManager>) {
    log(LogLevel::Info, "Applying new configuration...");
    manager.update_projects(config.projects).await;
}

// A helper function to combine loading and applying the configuration.
pub async fn load_and_apply_config(manager: Arc<PathmapManager>) -> Result<(), AppError> {
    let config = load_config().await?;
    apply_config(config, manager).await;
    Ok(())
}

// Watches the config file for changes and sends an event through the channel.
pub async fn watch_config(tx: Sender<()>, manager: Arc<PathmapManager>) {
    let (watcher_tx, mut watcher_rx) = mpsc::channel(1);

    let mut watcher = match notify::recommended_watcher(move |res| {
        if let Ok(event) = res {
            if let Err(e) = watcher_tx.blocking_send(event) {
                log(
                    LogLevel::Error,
                    &format!("Failed to send watcher event: {}", e),
                );
            }
        }
    }) {
        Ok(w) => w,
        Err(e) => {
            log(
                LogLevel::Error,
                &format!("Failed to create file watcher: {}", e),
            );
            return;
        }
    };

    if let Err(e) = watcher.watch(Path::new(CONFIG_PATH), RecursiveMode::NonRecursive) {
        log(
            LogLevel::Error,
            &format!("Failed to watch config file {}: {}", CONFIG_PATH, e),
        );
        return;
    }

    log(
        LogLevel::Info,
        &format!("Watching for changes in {}", CONFIG_PATH),
    );

    while let Some(event) = watcher_rx.recv().await {
        match event.kind {
            EventKind::Modify(_) | EventKind::Create(_) => {
                log(
                    LogLevel::Info,
                    "Config file changed. Sending update signal.",
                );
                if let Err(e) = load_and_apply_config(manager.clone()).await {
                    log(LogLevel::Error, &format!("Failed to reload config: {}", e));
                }
                if tx.send(()).await.is_err() {
                    log(
                        LogLevel::Error,
                        "Failed to send config update signal: receiver dropped.",
                    );
                    break;
                }
            }
            _ => {} // Ignore other events
        }
    }
}
