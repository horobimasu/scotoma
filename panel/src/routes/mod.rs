mod start_build;
mod update_config;

use axum::Router;
use axum::routing::post;

use start_build::start_build;
use update_config::update_config;

pub fn router() -> Router {
    Router::new()
        .route("/start-build", post(start_build))
        .route("/update-config", post(update_config))
}
