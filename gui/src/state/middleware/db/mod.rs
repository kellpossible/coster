//! So, init -> LoadStore action -> figures out what items to read,
//! how to read them, dispatches change events for all items that were
//! read.

mod dispatch;

pub use dispatch::DatabaseDispatch;
use kvdb::KeyValueDB;
use std::rc::Rc;
use yew_state::{middleware::Middleware, Store};

pub struct DatabaseMiddleware<DB> {
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

#[derive(Clone)]
pub struct DatabaseEffect<State, Action, Event, Effect>(
    Rc<dyn Fn(&Store<State, Action, Event, Effect>, &dyn KeyValueDB)>,
);

impl<State, Action, Event, Effect> DatabaseEffect<State, Action, Event, Effect> {
    pub fn run(&self, store: &Store<State, Action, Event, Effect>, db: &dyn KeyValueDB) {
        (self.0)(store, db)
    }
}

impl<F, State, Action, Event, Effect> From<F> for DatabaseEffect<State, Action, Event, Effect>
where
    F: Fn(&Store<State, Action, Event, Effect>, &dyn KeyValueDB) + 'static,
{
    fn from(f: F) -> Self {
        DatabaseEffect(Rc::new(f))
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
            db_effect.run(store, &self.database);
            None
        } else {
            Some(effect)
        }
    }
}
