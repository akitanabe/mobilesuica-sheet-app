use rand::distributions::{Alphanumeric, DistString};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Mutex, MutexGuard, OnceLock},
};

type SessionStore = Mutex<HashMap<String, Session>>;
static SESSION_STORE: OnceLock<SessionStore> = OnceLock::new();

static SESSION_EXPIRED_TIME: u64 = match cfg!(test) {
    true => 1,                 // テスト時は1秒
    false => 60 * 60 * 24 * 1, // 本番時は1日
};

#[derive(Clone, Debug, Default)]
pub struct Session {
    id: String,
    data: HashMap<String, String>,
    expired_at: u64,
}

fn genereate_session_id() -> String {
    let mut rng = rand::thread_rng();
    Alphanumeric.sample_string(&mut rng, 32)
}

fn get_session_store<'a>() -> &'a SessionStore {
    SESSION_STORE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn clear_session(session_store: &mut MutexGuard<HashMap<String, Session>>, session_id: &str) -> () {
    if let Some(session) = session_store.remove(session_id) {
        drop(session);
    }
}

impl Session {
    pub fn new() -> String {
        let session_store = get_session_store();

        let session_id = genereate_session_id();

        let expired_at = (chrono::Local::now().timestamp() as u64) + SESSION_EXPIRED_TIME;

        session_store.lock().unwrap().insert(
            session_id.clone(),
            Session {
                id: session_id.clone(),
                data: HashMap::new(),
                expired_at,
            },
        );

        session_id
    }

    pub fn get_session(session_id: &str) -> Option<Session> {
        // セッションストアのロックで期限が更新できなくなるため
        // session_storeの所有権を放棄するためにスコープを分ける
        let mut session = {
            let mut session_store = get_session_store().lock().unwrap();
            let session = session_store.get(session_id)?;

            if session.is_expired() {
                clear_session(&mut session_store, session_id);
                return None;
            }
            session.clone()
        };

        // セッションの有効期限を更新
        session.update_expired_at();
        session.save();

        Some(session)
    }

    pub fn has_session(session_id: &str) -> bool {
        let mut session_store = get_session_store().lock().unwrap();

        if let Some(session) = session_store.get(session_id) {
            if session.is_expired() {
                clear_session(&mut session_store, session_id);
                return false;
            }

            true
        } else {
            false
        }
    }

    pub fn gc() -> () {
        {
            let session_store = get_session_store().lock().unwrap();

            session_store.keys().cloned().collect::<Vec<String>>()
        }
        .iter()
        .for_each(|session_id| {
            let mut session_store = get_session_store().try_lock().unwrap();

            if let Some(session) = session_store.get(session_id) {
                if session.is_expired() {
                    clear_session(&mut session_store, session_id);
                }
            }
        });
    }

    pub fn set<T>(&mut self, key: &str, value: T) -> ()
    where
        T: Serialize,
    {
        let serialized = serde_json::to_string(&value).unwrap();
        self.data.insert(key.to_string(), serialized);

        self.save();
    }

    pub fn get<T>(&self, key: &str) -> Option<T>
    where
        T: for<'a> Deserialize<'a>,
    {
        let serialzied = self.data.get(key)?.clone();

        let deserialized: T = serde_json::from_str(&serialzied).ok()?;

        return Some(deserialized);
    }

    pub fn save(&self) -> () {
        get_session_store()
            .lock()
            .unwrap()
            .insert(self.id.clone(), self.clone());
    }

    pub fn clear(&mut self) -> () {
        if let Ok(mut session_store) = get_session_store().try_lock() {
            clear_session(&mut session_store, &self.id);
        }
    }

    pub fn is_expired(&self) -> bool {
        let time = chrono::Local::now().timestamp() as u64;

        if self.expired_at < time {
            return true;
        }

        false
    }

    pub fn update_expired_at(&mut self) -> () {
        self.expired_at = (chrono::Local::now().timestamp() as u64) + SESSION_EXPIRED_TIME;
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

        let created_session = Session::get_session(&session_id).unwrap();

        assert_eq!(created_session.get::<String>("test").unwrap(), "test");
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

        let test: Test = session.get::<Test>("test_key").unwrap();

        assert_eq!(test.prop, "test1");
    }

    #[test]
    #[should_panic]
    fn test_session_get_empty() {
        let session_id = Session::new();

        let mut session = Session::get_session(&session_id).unwrap();

        session.set("test", "test");

        let session = Session::get_session(&session_id).unwrap();

        session.get::<String>("test2").unwrap();
    }

    #[test]
    #[should_panic]
    fn test_session_clear() {
        let session_id = Session::new();

        let mut session = Session::get_session(&session_id).unwrap();

        session.set("test", "test");
        session.clear();

        let cleared_session = Session::get_session(&session_id);

        cleared_session.unwrap();
    }

    #[test]
    fn test_session_has() {
        let session_id = Session::new();

        assert_eq!(Session::has_session(&session_id), true);
        assert_eq!(Session::has_session(""), false);
    }

    #[tokio::test]
    async fn test_session_gc() {
        let session_id = Session::new();

        let mut session = Session::get_session(&session_id).unwrap();

        session.set("test", "test");
        // GCが行われると全ての期限切れSessionが廃棄されてしまうので他のテストとの兼ね合いで3秒待つ
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

        Session::gc();

        let session = Session::get_session(&session_id);

        assert_eq!(session.is_none(), true);
    }

    #[tokio::test]
    async fn test_session_get_update_expired_at() {
        let session_id = Session::new();

        let mut session = Session::get_session(&session_id).unwrap();

        session.set("test", "test");
        // 1秒経過時には期限内なので更新されている
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        let session = Session::get_session(&session_id);

        assert_eq!(session.is_some(), true);

        // 2秒経過時には期限外なので更新されていない
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        let session = Session::get_session(&session_id);

        assert_eq!(session.is_none(), true);
    }

    #[tokio::test]
    async fn test_session_update_expired_at() {
        let session_id = Session::new();

        let mut session = Session::get_session(&session_id).unwrap();

        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        session.update_expired_at();
        session.save();

        let session = Session::get_session(&session_id);

        assert_eq!(session.is_some(), true);
    }
}
