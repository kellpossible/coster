//! So, init -> LoadStore action -> figures out what items to read,
//! how to read them, dispatches change events for all items that were
//! read.

mod dispatch;

pub use dispatch::DatabaseDispatch;
use kvdb::KeyValueDB;
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, hash::Hash};
use yew_state::{middleware::Middleware, Store};

struct DatabaseMiddleware<DB> {
    database: DB,
    /// Whether or not the middleware is currently invoking an action
    /// to write to the store the result of a read from the database.
    reading_database: bool,
}

impl<DB> DatabaseMiddleware<DB>
where
    DB: KeyValueDB,
{
    pub fn new(database: DB) -> Self {
        Self {
            database,
            reading_database: false,
        }
    }
}

enum DatabaseAction {
    LoadStore,
}

trait IsDatabaseAction {
    fn database_action(&self) -> Option<DatabaseAction>;
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum DatabaseEvent<DB, State, Action, Event> {
    #[serde(skip_deserializing)]
    Dispatch(DatabaseDispatch<DB, State, Action, Event>),
}

trait IsDatabaseEvent<DB, State, Action, Event> {
    fn database_event(&self) -> Option<DatabaseEvent<DB, State, Action, Event>>;
}

impl<DB, State, Action, Event> Middleware<State, Action, Event> for DatabaseMiddleware<DB>
where
    Action: IsDatabaseAction,
    Event: IsDatabaseEvent<DB, State, Action, Event>,
{
    fn on_reduce(
        &self,
        store: &Store<State, Action, Event>,
        action: Option<Action>,
        reduce: yew_state::middleware::ReduceFn<State, Action, Event>,
    ) -> Vec<Event> {
        reduce(store, action)
    }

    fn on_notify(
        &self,
        store: &Store<State, Action, Event>,
        events: Vec<Event>,
        notify: yew_state::middleware::NotifyFn<State, Action, Event>,
    ) -> Vec<Event> {
        for database_event in events.iter().filter_map(|e| e.database_event()) {
            match database_event {
                DatabaseEvent::Dispatch(dispatch) => {
                    dispatch.run(store, &self.database, self.reading_database)
                }
            }
        }
        notify(store, events)
    }
}
