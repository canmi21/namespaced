/* src/api.rs */

use crate::{error::AppError, manager::PathmapManager};
use axum::{
    Json,
    Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, post, put}, // Corrected: Removed unused IntoResponse
};
use serde_json::Value;
use std::sync::Arc;

type AppState = State<Arc<PathmapManager>>;

// Creates the main router for the application.
pub fn create_router(manager: Arc<PathmapManager>) -> Router {
    Router::new()
        .route("/:project/:path", get(get_value))
        .route("/:project/:path", post(set_value))
        .route("/:project/:path", put(overwrite_value))
        .route("/:project/:path", delete(delete_value))
        .route("/exists/:project/:path", get(check_existence))
        .with_state(manager)
}

// GET /<project>/<path>
async fn get_value(
    State(manager): AppState,
    Path((project, path)): Path<(String, String)>,
) -> Result<Json<Value>, AppError> {
    let value: Value = manager.get(&project, &path).await?;
    Ok(Json(value))
}

// POST /<project>/<path>
async fn set_value(
    State(manager): AppState,
    Path((project, path)): Path<(String, String)>,
    Json(payload): Json<Value>,
) -> Result<StatusCode, AppError> {
    manager.set(&project, &path, &payload).await?;
    Ok(StatusCode::CREATED)
}

// PUT /<project>/<path>
async fn overwrite_value(
    State(manager): AppState,
    Path((project, path)): Path<(String, String)>,
    Json(payload): Json<Value>,
) -> Result<StatusCode, AppError> {
    manager.overwrite(&project, &path, &payload).await?;
    Ok(StatusCode::OK)
}

// DELETE /<project>/<path>
async fn delete_value(
    State(manager): AppState,
    Path((project, path)): Path<(String, String)>,
) -> Result<StatusCode, AppError> {
    manager.delete(&project, &path).await?;
    Ok(StatusCode::NO_CONTENT)
}

// GET /exists/<project>/<path>
async fn check_existence(
    State(manager): AppState,
    Path((project, path)): Path<(String, String)>,
) -> Result<StatusCode, AppError> {
    let exists = manager.exists(&project, &path).await?;
    if exists {
        Ok(StatusCode::OK)
    } else {
        Err(AppError::NotFound(format!("{}::{}", project, path)))
    }
}
