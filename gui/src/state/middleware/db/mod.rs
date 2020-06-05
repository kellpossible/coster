//! So, init -> LoadStore action -> figures out what items to read,
//! how to read them, dispatches change events for all items that were
//! read.

mod dispatch;

pub use dispatch::DatabaseDispatch;
use kvdb::KeyValueDB;
use yew_state::middleware::Middleware;

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

impl<DB, State, Action, Event, Effect> Middleware<State, Action, Event, Effect>
    for DatabaseMiddleware<DB>
where
    Action: IsDatabaseAction,
{
}
