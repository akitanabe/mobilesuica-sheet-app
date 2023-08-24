use mobilesuica_sheet_app_server::Session;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Default)]
pub struct AppState {
    pub session: Arc<Mutex<Session>>,
}
