use rand::distributions::{Alphanumeric, DistString};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
};

static SESSION_STORE: OnceLock<Mutex<HashMap<String, Session>>> = OnceLock::new();

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

        if String::is_empty(&serialzied) {
            return T::default();
        }

        let deserialized: T = serde_json::from_str(&serialzied).unwrap_or_default();

        return deserialized;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_session_create() {
        let session_id = Session::new();

        let mut session = Session::get_session(&session_id);

        session.set("test", "test");

        let session = Session::get_session(&session_id);

        assert_eq!(session.get::<String>("test"), "test");
    }

    #[test]
    fn test_session_set() {
        #[derive(Deserialize, Serialize, Default)]
        struct Test {
            pub prop: String,
        }

        let session_id = Session::new();

        let mut session = Session::get_session(&session_id);

        session.set(
            "test_key",
            Test {
                prop: "test1".to_string(),
            },
        );

        let test: Test = session.get::<Test>("test_key");

        assert_eq!(test.prop, "test1");
    }

    #[test]
    fn test_session_get_empty() {
        let session_id = Session::new();

        let mut session = Session::get_session(&session_id);

        session.set("test", "test");

        let session = Session::get_session(&session_id);

        assert_eq!(session.get::<String>("test_empty"), "");
    }
}
