use axum::{routing::get, Router};
mod captcha;

pub fn router() -> Router {
    Router::new().route("/captcha", get(captcha::handler))
}
