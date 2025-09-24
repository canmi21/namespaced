/* src/api.rs */

use crate::{AppState, admin, error::AppError};
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, put},
};
use serde_json::Value;
use std::sync::Arc;

// This type alias makes the handler signatures cleaner
type AppStateExtractor = State<Arc<AppState>>;

// Creates the main router for the application.
pub fn create_router(state: Arc<AppState>) -> Router {
    // Router for administrative tasks like managing projects
    let admin_router = Router::new()
        .route(
            "/_namespaced/projects",
            get(admin::list_projects).post(admin::create_project),
        )
        // CORRECTED: Used `{project}` instead of the old `:project` syntax.
        .route(
            "/_namespaced/projects/{project}",
            put(admin::update_project).delete(admin::delete_project),
        );

    // Router for core data operations
    let data_router = Router::new()
        .route(
            "/{project}/{path}",
            get(get_value)
                .post(set_value)
                .put(overwrite_value)
                .delete(delete_value),
        )
        .route("/exists/{project}/{path}", get(check_existence));

    // Combine all routers and apply the shared state, with specific routes first.
    Router::new()
        .merge(admin_router)
        .merge(data_router)
        .with_state(state)
}

// --- Data handler functions remain the same ---

async fn get_value(
    State(state): AppStateExtractor,
    Path((project, path)): Path<(String, String)>,
) -> Result<Json<Value>, AppError> {
    let value: Value = state.manager.get(&project, &path).await?;
    Ok(Json(value))
}

async fn set_value(
    State(state): AppStateExtractor,
    Path((project, path)): Path<(String, String)>,
    Json(payload): Json<Value>,
) -> Result<StatusCode, AppError> {
    state.manager.set(&project, &path, &payload).await?;
    Ok(StatusCode::CREATED)
}

async fn overwrite_value(
    State(state): AppStateExtractor,
    Path((project, path)): Path<(String, String)>,
    Json(payload): Json<Value>,
) -> Result<StatusCode, AppError> {
    state.manager.overwrite(&project, &path, &payload).await?;
    Ok(StatusCode::OK)
}

async fn delete_value(
    State(state): AppStateExtractor,
    Path((project, path)): Path<(String, String)>,
) -> Result<StatusCode, AppError> {
    state.manager.delete(&project, &path).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn check_existence(
    State(state): AppStateExtractor,
    Path((project, path)): Path<(String, String)>,
) -> Result<StatusCode, AppError> {
    let exists = state.manager.exists(&project, &path).await?;
    if exists {
        Ok(StatusCode::OK)
    } else {
        Err(AppError::NotFound(format!("{}::{}", project, path)))
    }
}
