/// Extension/composition test — equivalent to lowdb's Lodash test.
///
/// Demonstrates that rufdb's `Low` and `LowSync` can be wrapped and
/// extended with additional functionality, similar to how lowdb supports
/// `class LowWithLodash<T> extends Low<T>`.
///
/// In Rust, the idiomatic equivalent is wrapping in a newtype struct.
use rufdb::{Low, LowSync, Memory, MemorySync};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq)]
struct Data {
    todos: Vec<String>,
}

/// A wrapper around `LowSync` that adds query-like methods,
/// similar to how lowdb users extend Low with lodash chains.
struct LowWithQuery<T, A>
where
    T: Send + Sync,
    A: rufdb::SyncAdapter<T>,
{
    db: LowSync<T, A>,
}

impl<A> LowWithQuery<Data, A>
where
    A: rufdb::SyncAdapter<Data>,
{
    fn new(adapter: A, default_data: Data) -> Self {
        Self {
            db: LowSync::new(adapter, default_data),
        }
    }

    /// Get the first todo (like lodash's `chain.get('todos').first().value()`)
    fn first_todo(&self) -> Option<&String> {
        self.db.data.todos.first()
    }

    /// Find todos containing a substring
    fn find_todos(&self, needle: &str) -> Vec<&String> {
        self.db
            .data
            .todos
            .iter()
            .filter(|t| t.contains(needle))
            .collect()
    }

    /// Add a todo and write
    fn add_todo(&mut self, todo: String) -> Result<(), rufdb::RufDbError> {
        self.db.update(|data| {
            data.todos.push(todo);
        })
    }
}

#[test]
fn extension_wrapper_query_methods() {
    let adapter = MemorySync::new();
    let default = Data {
        todos: vec!["buy milk".into(), "write code".into(), "buy eggs".into()],
    };
    let wrapper = LowWithQuery::new(adapter, default);

    // first_todo (like lodash chain.get('todos').first().value())
    assert_eq!(wrapper.first_todo(), Some(&"buy milk".to_string()));

    // find_todos (like lodash filter)
    let buy_items = wrapper.find_todos("buy");
    assert_eq!(buy_items.len(), 2);
    assert_eq!(buy_items[0], "buy milk");
    assert_eq!(buy_items[1], "buy eggs");
}

#[test]
fn extension_wrapper_add_and_persist() {
    let adapter = MemorySync::new();
    let mut wrapper = LowWithQuery::new(adapter, Data::default());

    wrapper.add_todo("first".into()).unwrap();
    wrapper.add_todo("second".into()).unwrap();

    assert_eq!(wrapper.db.data.todos.len(), 2);
    assert_eq!(wrapper.first_todo(), Some(&"first".to_string()));
}

/// Async wrapper — same pattern but with Low instead of LowSync.
struct AsyncLowWithQuery<T, A>
where
    T: Send + Sync,
    A: rufdb::Adapter<T>,
{
    db: Low<T, A>,
}

impl<A> AsyncLowWithQuery<Data, A>
where
    A: rufdb::Adapter<Data>,
{
    fn new(adapter: A, default_data: Data) -> Self {
        Self {
            db: Low::new(adapter, default_data),
        }
    }

    fn first_todo(&self) -> Option<&String> {
        self.db.data.todos.first()
    }

    async fn add_todo(&mut self, todo: String) -> Result<(), rufdb::RufDbError> {
        self.db
            .update(|data| {
                data.todos.push(todo);
            })
            .await
    }
}

#[tokio::test]
async fn async_extension_wrapper() {
    let adapter = Memory::new();
    let default = Data {
        todos: vec!["foo".into(), "bar".into()],
    };
    let mut wrapper = AsyncLowWithQuery::new(adapter, default);

    assert_eq!(wrapper.first_todo(), Some(&"foo".to_string()));

    wrapper.add_todo("baz".into()).await.unwrap();
    assert_eq!(wrapper.db.data.todos.len(), 3);
}
