use rand::distributions::{Alphanumeric, DistString};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
};

type SessionStore = Mutex<HashMap<String, Session>>;
static SESSION_STORE: OnceLock<SessionStore> = OnceLock::new();

#[derive(Clone, Debug, Default)]
pub struct Session {
    id: String,
    data: HashMap<String, String>,
}

fn genereate_session_id() -> String {
    let mut rng = rand::thread_rng();
    Alphanumeric.sample_string(&mut rng, 32)
}

fn get_session_store<'a>() -> &'a SessionStore {
    SESSION_STORE.get_or_init(|| Mutex::new(HashMap::new()))
}

impl Session {
    pub fn new() -> String {
        let session_store = get_session_store();

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

    pub fn get_session(session_id: &str) -> Option<Session> {
        let session_store = get_session_store().lock().unwrap();
        let session = session_store.get(session_id);

        match session {
            Some(session) => Some(session.clone()),
            None => None,
        }
    }

    pub fn has_session(session_id: &str) -> bool {
        get_session_store().lock().unwrap().contains_key(session_id)
    }

    pub fn set<T>(&mut self, key: &str, value: T) -> ()
    where
        T: Serialize,
    {
        let serialized = serde_json::to_string(&value).unwrap();
        self.data.insert(key.to_string(), serialized);

        get_session_store()
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

    pub fn clear(&mut self) -> () {
        self.data.clear();

        get_session_store().lock().unwrap().remove(&self.id);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_session_create() {
        let session_id = Session::new();

        let mut session = Session::get_session(&session_id).unwrap();

        session.set("test", "test");

        let session = Session::get_session(&session_id).unwrap();

        assert_eq!(session.get::<String>("test"), "test");
    }

    #[test]
    fn test_session_set() {
        #[derive(Deserialize, Serialize, Default)]
        struct Test {
            pub prop: String,
        }

        let session_id = Session::new();

        let mut session = Session::get_session(&session_id).unwrap();

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

        let mut session = Session::get_session(&session_id).unwrap();

        session.set("test", "test");

        let session = Session::get_session(&session_id).unwrap();

        assert_eq!(session.get::<String>("test_empty"), "");
    }

    #[test]
    fn test_session_clear() {
        let session_id = Session::new();

        let mut session = Session::get_session(&session_id).unwrap();

        session.set("test", "test");
        session.clear();

        let cleared_session = Session::get_session(&session_id).expect("session is cleared");

        assert_eq!(cleared_session.get::<String>("test"), "session is cleared");
    }

    #[test]
    fn test_session_has() {
        let session_id = Session::new();

        assert_eq!(Session::has_session(&session_id), true);
        assert_eq!(Session::has_session(""), false);
    }
}
