use axum::{routing::get, Router};

use crate::AppState;

mod auth;
mod captcha;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/captcha", get(captcha::handler))
        .route("/auth", get(auth::handler))
}
