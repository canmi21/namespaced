/* src/manager.rs */

use dashmap::DashMap;
use fancy_log::{LogLevel, log};
use pathmap::Pathmap;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::mpsc::Receiver;

use crate::error::AppError;

// Manages all active Pathmap instances.
pub struct PathmapManager {
    // Using DashMap for thread-safe concurrent access.
    instances: DashMap<String, Arc<Pathmap>>,
}

impl PathmapManager {
    pub fn new() -> Self {
        Self {
            instances: DashMap::new(),
        }
    }

    // Updates the running instances based on the new configuration.
    pub async fn update_projects(&self, new_projects: HashMap<String, String>) {
        let mut projects_to_add = new_projects.clone();

        self.instances.retain(|project_name, _| {
            if !new_projects.contains_key(project_name) {
                log(
                    LogLevel::Info,
                    &format!("Removing project: {}", project_name),
                );
                false
            } else {
                projects_to_add.remove(project_name);
                true
            }
        });

        for (name, path) in projects_to_add {
            log(
                LogLevel::Info,
                &format!("Adding project '{}' with path '{}'", name, path),
            );
            let pm = Pathmap::new().with_base_path(&path);
            self.instances.insert(name, Arc::new(pm));
        }
    }

    fn get_instance(&self, project: &str) -> Result<Arc<Pathmap>, AppError> {
        self.instances
            .get(project)
            .map(|entry| entry.value().clone())
            .ok_or_else(|| AppError::ProjectNotFound(project.to_string()))
    }

    // --- API Methods ---

    // Corrected: Manually map pathmap's error to our AppError string variant.
    pub async fn get<T: DeserializeOwned>(&self, project: &str, path: &str) -> Result<T, AppError> {
        let pm = self.get_instance(project)?;
        pm.get(path)
            .await
            .map_err(|e| AppError::Pathmap(e.to_string()))
    }

    pub async fn set<T: Serialize + Send + Sync>(
        &self,
        project: &str,
        path: &str,
        value: &T,
    ) -> Result<(), AppError> {
        let pm = self.get_instance(project)?;
        pm.set(path, value)
            .await
            .map_err(|e| AppError::Pathmap(e.to_string()))
    }

    pub async fn overwrite<T: Serialize + Send + Sync>(
        &self,
        project: &str,
        path: &str,
        value: &T,
    ) -> Result<(), AppError> {
        let pm = self.get_instance(project)?;
        pm.overwrite(path, value)
            .await
            .map_err(|e| AppError::Pathmap(e.to_string()))
    }

    pub async fn delete(&self, project: &str, path: &str) -> Result<(), AppError> {
        let pm = self.get_instance(project)?;
        pm.delete(path)
            .await
            .map_err(|e| AppError::Pathmap(e.to_string()))
    }

    pub async fn exists(&self, project: &str, path: &str) -> Result<bool, AppError> {
        let pm = self.get_instance(project)?;
        pm.exists(path)
            .await
            .map_err(|e| AppError::Pathmap(e.to_string()))
    }
}

pub async fn handle_config_updates(mut rx: Receiver<()>, manager: Arc<PathmapManager>) {
    while rx.recv().await.is_some() {
        log(LogLevel::Info, "Received config update signal. Reloading.");
        if let Err(e) = crate::config::load_and_apply_config(manager.clone()).await {
            log(LogLevel::Error, &format!("Failed to reload config: {}", e));
        }
    }
}
