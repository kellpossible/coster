//! So, init -> LoadStore action -> figures out what items to read,
//! how to read them, dispatches change events for all items that were
//! read.

mod dispatch;

pub use dispatch::DatabaseDispatch;
use kvdb::{DBTransaction, KeyValueDB};
use serde::{de::DeserializeOwned, Serialize};
use std::{io, rc::Rc, cell::RefCell};
use yew_state::{middleware::Middleware, Store};

pub struct DatabaseMiddleware<DB> {
    database: DB,
    // whether or not the database is blocking effects from other actions.
    blocked: RefCell<bool>,
}

impl<DB> DatabaseMiddleware<DB>
where
    DB: KeyValueDB,
{
    pub fn new(database: DB) -> Self {
        Self {
            database,
            blocked: RefCell::new(false),
        }
    }
}

enum DatabaseAction {
    LoadStore,
}

pub trait KeyValueDBSerde {
    fn get_deserialize<K: AsRef<str>, V: DeserializeOwned>(
        &self,
        col: u32,
        key: K,
    ) -> io::Result<Option<V>>;
}

pub trait DBTransactionSerde {
    fn put_serialize<K: AsRef<str>, V: Serialize>(&mut self, col: u32, key: K, value: V);
}

impl KeyValueDBSerde for &dyn KeyValueDB
{
    fn get_deserialize<K: AsRef<str>, V: DeserializeOwned>(
        &self,
        col: u32,
        key: K,
    ) -> io::Result<Option<V>> {
        self.get(col, key.as_ref().as_bytes()).map(|value_option| {
            value_option.map(|value_bytes| {
                serde_json::from_slice(&value_bytes).expect("unable to desrialize database value")
            })
        })
    }
}

impl DBTransactionSerde for DBTransaction {
    fn put_serialize<K: AsRef<str>, V: Serialize>(&mut self, col: u32, key: K, value: V) {
        let value_string =
            serde_json::to_string(&value).expect("unable to serialize database value");

        self.put(col, key.as_ref().as_bytes(), value_string.as_bytes())
    }
}

// TODO: this could be refactored into an enum, with effect for read, write, and then custom closure.
// Would make it easier to debug this code with the logger, and more explicit about what is going on.
// Custom closure could have a name too.
#[derive(Clone)]
pub struct DatabaseEffect<State, Action, Event, Effect>{
    closure: Rc<dyn Fn(&Store<State, Action, Event, Effect>, &dyn KeyValueDB)>,
    pub blocking: bool,
}

impl<State, Action, Event, Effect> DatabaseEffect<State, Action, Event, Effect> {
    pub fn new<F>(f: F, blocking: bool) -> Self
    where
        F: Fn(&Store<State, Action, Event, Effect>, &dyn KeyValueDB) + 'static,
    {
        DatabaseEffect {
            closure: Rc::new(f),
            blocking
        }
    }

    pub fn run(&self, store: &Store<State, Action, Event, Effect>, db: &dyn KeyValueDB) {
        (self.closure)(store, db)
    }
}

pub trait IsDatabaseEffect<State, Action, Event, Effect> {
    fn database_effect(&self) -> Option<&DatabaseEffect<State, Action, Event, Effect>>;
}

trait IsDatabaseAction {
    fn database_action(&self) -> Option<DatabaseAction>;
}

impl<DB, State, Action, Event, Effect> Middleware<State, Action, Event, Effect>
    for DatabaseMiddleware<DB>
where
    DB: KeyValueDB,
    Effect: IsDatabaseEffect<State, Action, Event, Effect>,
{
    fn on_reduce(
        &self,
        store: &Store<State, Action, Event, Effect>,
        action: Option<&Action>,
        reduce: yew_state::middleware::ReduceFn<State, Action, Event, Effect>,
    ) -> yew_state::middleware::ReduceMiddlewareResult<Event, Effect> {
        reduce(store, action)
    }

    fn process_effect(
        &self,
        store: &Store<State, Action, Event, Effect>,
        effect: Effect,
    ) -> Option<Effect> {
        if let Some(db_effect) = effect.database_effect() {
            // TODO: there is a bug with this blocking code, it doesn't work with asynchronous actions, and actions
            // executed later. Perhaps better not to do with blocking, and instead add a flag to the action to indicate
            // whether or not it should trigger a database action.
            // If I reformat the effect code logic to be seperated, it could be cleaner...
            let blocked = *self.blocked.borrow();
            log::debug!("DatabaseMiddleware::process_effect blocked: {}", blocked);
            if !blocked {
                log::debug!("DatabaseMiddleware::process_effect effect.blocking: {}", db_effect.blocking);
                *self.blocked.borrow_mut() = db_effect.blocking;
                db_effect.run(store, &self.database);
                log::debug!("DatabaseMiddleware::process_effect after run");
                if db_effect.blocking {
                    *self.blocked.borrow_mut() = false;
                }
            }
            
            None
        } else {
            Some(effect)
        }
    }
}
