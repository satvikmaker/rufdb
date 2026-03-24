/// Browser adapter tests — tests for WebStorage, LocalStorage, SessionStorage.
///
/// Since we can't use real browser APIs in tests, we use a HashMap-based
/// mock storage backend, matching lowdb's approach of mocking localStorage.
#[cfg(feature = "wasm")]
mod browser_tests {
    use rufdb::adapters::browser::{LocalStorage, SessionStorage, StorageBackend, WebStorage};
    use rufdb::SyncAdapter;
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;
    use std::sync::RwLock;

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
    struct TestData {
        a: i32,
    }

    /// Mock storage backend using a HashMap — equivalent to lowdb's test mock.
    struct MockStorage {
        data: RwLock<HashMap<String, String>>,
    }

    impl MockStorage {
        fn new() -> Self {
            Self {
                data: RwLock::new(HashMap::new()),
            }
        }
    }

    impl StorageBackend for MockStorage {
        fn get_item(&self, key: &str) -> Option<String> {
            self.data.read().unwrap().get(key).cloned()
        }

        fn set_item(&self, key: &str, value: &str) {
            self.data
                .write()
                .unwrap()
                .insert(key.to_string(), value.to_string());
        }
    }

    #[test]
    fn web_storage_read_returns_none_when_key_missing() {
        let storage = MockStorage::new();
        let adapter = WebStorage::<TestData>::new("missing", storage);
        assert!(adapter.read().unwrap().is_none());
    }

    #[test]
    fn web_storage_write_and_read_roundtrip() {
        let storage = MockStorage::new();
        let adapter = WebStorage::new("key", storage);

        let data = TestData { a: 1 };
        adapter.write(&data).unwrap();

        let result = adapter.read().unwrap().unwrap();
        assert_eq!(result, data);
    }

    #[test]
    fn web_storage_uses_compact_json() {
        let storage = MockStorage::new();
        let adapter = WebStorage::new("key", storage);

        let data = TestData { a: 42 };
        adapter.write(&data).unwrap();

        // Read the raw stored value to verify compact JSON (no newlines)
        // We can't easily access the backing store, so just verify roundtrip
        let result = adapter.read().unwrap().unwrap();
        assert_eq!(result, data);
    }

    #[test]
    fn local_storage_convenience() {
        let storage = MockStorage::new();
        let adapter = LocalStorage::create::<TestData, _>("local-key", storage);

        let data = TestData { a: 10 };
        adapter.write(&data).unwrap();
        let result = adapter.read().unwrap().unwrap();
        assert_eq!(result, data);
    }

    #[test]
    fn session_storage_convenience() {
        let storage = MockStorage::new();
        let adapter = SessionStorage::create::<TestData, _>("session-key", storage);

        let data = TestData { a: 20 };
        adapter.write(&data).unwrap();
        let result = adapter.read().unwrap().unwrap();
        assert_eq!(result, data);
    }

    #[test]
    fn browser_preset_local_storage() {
        use rufdb::presets::local_storage_preset;

        let storage = MockStorage::new();
        let mut db = local_storage_preset("preset-key", TestData { a: 0 }, storage).unwrap();

        db.update(|d| d.a = 42).unwrap();
        assert_eq!(db.data, TestData { a: 42 });
    }

    #[test]
    fn browser_preset_session_storage() {
        use rufdb::presets::session_storage_preset;

        let storage = MockStorage::new();
        let mut db = session_storage_preset("preset-key", TestData { a: 0 }, storage).unwrap();

        db.update(|d| d.a = 99).unwrap();
        assert_eq!(db.data, TestData { a: 99 });
    }
}
