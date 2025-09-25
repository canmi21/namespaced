/* src/api.rs */

use crate::{AppState, admin, error::AppError};
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, put},
};
use pathmap::Listing;
use serde::Serialize;
use serde_json::Value;
use std::sync::Arc;

// This type alias makes the handler signatures cleaner
type AppStateExtractor = State<Arc<AppState>>;

/// A serializable representation of a path's contents for API responses.
#[derive(Serialize)]
pub struct ListingResponse {
    groups: Vec<String>,
    values: Vec<String>,
}

// This allows us to easily convert from the library's type to our API type.
impl From<Listing> for ListingResponse {
    fn from(listing: Listing) -> Self {
        ListingResponse {
            groups: listing.groups,
            values: listing.values,
        }
    }
}

// Creates the main router for the application.
pub fn create_router(state: Arc<AppState>) -> Router {
    // Admin router for managing the service itself. Remains prefixed.
    let admin_router = Router::new()
        .route(
            "/_namespaced/projects",
            get(admin::list_projects).post(admin::create_project),
        )
        .route(
            "/_namespaced/projects/{project}",
            put(admin::update_project).delete(admin::delete_project),
        );

    // Main API router with distinct top-level actions.
    let api_router = Router::new()
        // Listing routes are now at the root.
        .route("/ls/{project}", get(list_namespaces))
        .route("/ls/{project}/{path}", get(list_path_contents))
        // Existence checks are now at the root.
        .route("/exists/{project}/{path}", get(check_existence))
        // Data manipulation is now the only action under "/namespaced/".
        .route(
            "/namespaced/{project}/{path}",
            get(get_value)
                .post(set_value)
                .put(overwrite_value)
                .delete(delete_value),
        );

    // Combine all routers.
    Router::new()
        .merge(admin_router)
        .merge(api_router)
        .with_state(state)
}

// --- Handlers ---
// No changes are needed to the handler function bodies, only their documentation comments.

/// GET /ls/{project}
async fn list_namespaces(
    State(state): AppStateExtractor,
    Path(project): Path<String>,
) -> Result<Json<Vec<String>>, AppError> {
    let namespaces = state.manager.list_ns(&project).await?;
    Ok(Json(namespaces))
}

/// GET /ls/{project}/{path}
async fn list_path_contents(
    State(state): AppStateExtractor,
    Path((project, path)): Path<(String, String)>,
) -> Result<Json<ListingResponse>, AppError> {
    let listing = state.manager.list_path(&project, &path).await?;
    Ok(Json(listing.into()))
}

/// GET, POST, PUT, DELETE /namespaced/{project}/{path}
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

/// GET /exists/{project}/{path}
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
