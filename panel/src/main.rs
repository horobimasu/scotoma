mod routes;
mod utils;
mod private;
mod state;

use axum::Router;

use tower_http::services::ServeDir;

use tokio::net::TcpListener;

use shared::*;

const HOST: &str = "127.0.0.1";
const PORT: &str = "1107";

#[tokio::main]
async fn main() {
    state::init_state();

    let app = Router::new()
        .nest("/api", routes::router())
        .fallback_service(ServeDir::new("public"));

    let listener = TcpListener::bind(format!("{}:{}", HOST, PORT))
        .await
        .unwrap();

    println!("listening on http://{}:{}", HOST, PORT);

    axum::serve(listener, app)
        .await
        .unwrap();
}
