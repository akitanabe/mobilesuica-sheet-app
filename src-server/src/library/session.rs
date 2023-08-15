use rand::distributions::{Alphanumeric, DistString};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
};

static SESSION_STORE: OnceLock<Mutex<HashMap<String, Session>>> = OnceLock::new();

#[derive(Clone, Debug)]
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

    pub fn get_session(session_id: String) -> Session {
        SESSION_STORE
            .get()
            .unwrap()
            .lock()
            .unwrap()
            .get(&session_id)
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
    {
        let serialzied = self.data.get(key).map(|v| v.clone()).unwrap();
        let deserialized: T = serde_json::from_str(&serialzied).unwrap();

        return deserialized;
    }
}
