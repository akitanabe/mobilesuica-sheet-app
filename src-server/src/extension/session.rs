use axum::{
    extract::{Request, State},
    http::{self, header},
    middleware::Next,
    response::Response,
};
use rand::distributions::{Alphanumeric, DistString};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, OnceLock},
};

static SESSION_STORE: OnceLock<Mutex<HashMap<String, Session>>> = OnceLock::new();

const SESSION_ID_HEADER: &str = "x-session-id";

#[derive(Clone, Debug, Default)]
pub struct Session {
    id: String,
    data: HashMap<String, String>,
}

fn genereate_session_id() -> String {
    let mut rng = rand::thread_rng();
    Alphanumeric.sample_string(&mut rng, 32)
}

impl Session {
    pub fn new() -> String {
        let session_store = SESSION_STORE.get_or_init(|| Mutex::new(HashMap::new()));

        let session_id = genereate_session_id();

        session_store.lock().unwrap().insert(
            session_id.clone(),
            Session {
                id: session_id.clone(),
                data: HashMap::new(),
            },
        );

        session_id
    }

    pub fn get_session(session_id: &str) -> Session {
        SESSION_STORE
            .get()
            .unwrap()
            .lock()
            .unwrap()
            .get(session_id)
            .unwrap()
            .clone()
    }

    pub fn set<T>(&mut self, key: &str, value: T) -> ()
    where
        T: Serialize,
    {
        let serialized = serde_json::to_string(&value).unwrap();
        self.data.insert(key.to_string(), serialized);

        SESSION_STORE
            .get()
            .unwrap()
            .lock()
            .unwrap()
            .insert(self.id.clone(), self.clone());
    }

    pub fn get<T>(&self, key: &str) -> T
    where
        T: for<'a> Deserialize<'a>,
        T: Default,
    {
        let serialzied = self.data.get(key).map_or("".to_string(), |v| v.clone());
        let deserialized: T = serde_json::from_str(&serialzied).unwrap_or_default();

        return deserialized;
    }
}

#[derive(Debug, Clone, Default)]
pub struct AppState {
    pub session: Arc<Mutex<Session>>,
}

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
