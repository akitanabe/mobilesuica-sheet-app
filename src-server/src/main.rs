mod controllers;
mod middleware;
mod route;
mod store;

use axum::{middleware as axum_middleware, Router};
use middleware::session_middleware;
use store::AppState;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let state = AppState::default();

    let app = Router::new()
        .nest("/api", route::api_router())
        .route_layer(axum_middleware::from_fn_with_state(
            state.clone(),
            session_middleware,
        ))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8000")
        .await
        .unwrap();

    tracing::debug!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}
