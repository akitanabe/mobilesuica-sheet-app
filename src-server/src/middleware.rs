use crate::store::AppState;
use axum::{
    extract::{Request, State},
    http::{self, header},
    middleware::Next,
    response::Response,
};
use mobilesuica_sheet_app_server::Session;

const SESSION_ID_HEADER: &str = "x-session-id";

pub async fn session_middleware(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Response {
    let session_id = req
        .headers()
        .get(SESSION_ID_HEADER)
        .unwrap_or(&http::header::HeaderValue::from_static(""))
        .to_str()
        .ok()
        .map(|sid| match sid {
            "" => Session::new(),
            _ => sid.to_string(),
        })
        .unwrap();

    *state.session.lock().unwrap() = Session::get_session(&session_id);

    // レスポンスここから
    let mut response = next.run(req).await;

    response.headers_mut().insert(
        SESSION_ID_HEADER,
        header::HeaderValue::from_str(&session_id).unwrap(),
    );

    response
}
