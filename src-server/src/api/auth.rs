use axum::extract::State;
use axum::response::Response;
use mobilesuica_sheet_app_server::{MobilesuicaCookies, MobilesuicaFormParams};

use crate::AppState;

pub async fn handler(State(state): State<AppState>) -> Response {
    let session = state.session.lock().unwrap();

    let cookies = session.get::<MobilesuicaCookies>("cookies");
    let mut mobilesuica_form_params =
        session.get::<MobilesuicaFormParams>("mobilesuica_form_params");
    dbg!(cookies, mobilesuica_form_params);

    Response::default()
}
