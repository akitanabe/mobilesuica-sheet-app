use crate::controllers::{auth, captcha};

use crate::store::AppState;
use axum::{
    routing::{get, post},
    Router,
};

pub fn api_router() -> Router<AppState> {
    Router::new()
        .route("/captcha", get(captcha::handler))
        .route("/auth", post(auth::handler))
}
