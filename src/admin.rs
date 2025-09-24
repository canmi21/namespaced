/* src/admin.rs */

use crate::{AppState, config, error::AppError};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::Deserialize;
use serde_json::Value;
use std::sync::Arc;

// Payload for creating a new project.
#[derive(Deserialize)]
pub struct CreateProjectPayload {
    name: String,
    path: String,
}

// Payload for updating an existing project's path.
#[derive(Deserialize)]
pub struct UpdateProjectPayload {
    path: String,
}

// GET /_admin/projects
pub async fn list_projects(State(_state): State<Arc<AppState>>) -> Result<Json<Value>, AppError> {
    let cfg = config::load_config().await?;
    Ok(Json(serde_json::to_value(cfg.projects)?))
}

// POST /_admin/projects
pub async fn create_project(
    State(state): State<Arc<AppState>>,
    // CORRECTED: Removed the incorrect Path extractor.
    // The project name and path now come from the JSON payload.
    Json(payload): Json<CreateProjectPayload>,
) -> Result<StatusCode, AppError> {
    let _lock = state.config_lock.lock().await;

    let mut cfg = config::load_config().await?;

    // Get the project name from the payload now.
    if cfg.projects.contains_key(&payload.name) {
        return Err(AppError::AdminOperationFailed(format!(
            "Project '{}' already exists.",
            payload.name
        )));
    }

    cfg.projects.insert(payload.name, payload.path);
    config::save_config(&cfg).await?;

    Ok(StatusCode::CREATED)
}

// PUT /_admin/projects/{project}
pub async fn update_project(
    State(state): State<Arc<AppState>>,
    Path(project): Path<String>,
    Json(payload): Json<UpdateProjectPayload>, // Use the clearer name
) -> Result<StatusCode, AppError> {
    let _lock = state.config_lock.lock().await;

    let mut cfg = config::load_config().await?;

    if !cfg.projects.contains_key(&project) {
        return Err(AppError::ProjectNotFound(project));
    }

    cfg.projects.insert(project, payload.path);
    config::save_config(&cfg).await?;

    Ok(StatusCode::OK)
}

// DELETE /_admin/projects/{project}
pub async fn delete_project(
    State(state): State<Arc<AppState>>,
    Path(project): Path<String>,
) -> Result<StatusCode, AppError> {
    let _lock = state.config_lock.lock().await;

    let mut cfg = config::load_config().await?;

    if cfg.projects.remove(&project).is_none() {
        return Err(AppError::ProjectNotFound(project));
    }

    config::save_config(&cfg).await?;

    Ok(StatusCode::NO_CONTENT)
}
