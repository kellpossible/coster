//! So, init -> LoadStore action -> figures out what items to read,
//! how to read them, dispatches change events for all items that were
//! read.

mod dispatch;

pub use dispatch::DatabaseDispatch;
use kvdb::KeyValueDB;
use serde::Serialize;
use std::{fmt::Debug, rc::Rc};
use yew_state::{middleware::Middleware, Store};

pub struct DatabaseMiddleware<DB> {
    database: DB,
}

impl<DB> DatabaseMiddleware<DB>
where
    DB: KeyValueDB,
{
    pub fn new(database: DB) -> Self {
        Self { database }
    }
}

// TODO: this could be refactored into an enum, with effect for read, write, and then custom closure.
// Would make it easier to debug this code with the logger, and more explicit about what is going on.
// Custom closure could have a name too.
#[derive(Clone, Serialize)]
pub struct DatabaseEffect<State, Action, Event, Effect> {
    debug: String,
    #[serde(skip)]
    closure: Rc<dyn Fn(&Store<State, Action, Event, Effect>, &dyn KeyValueDB)>,
}

impl<State, Action, Event, Effect> DatabaseEffect<State, Action, Event, Effect> {
    pub fn new<F, S>(debug: S, f: F) -> Self
    where
        F: Fn(&Store<State, Action, Event, Effect>, &dyn KeyValueDB) + 'static,
        S: Into<String>,
    {
        DatabaseEffect {
            debug: debug.into(),
            closure: Rc::new(f),
        }
    }

    pub fn run(&self, store: &Store<State, Action, Event, Effect>, db: &dyn KeyValueDB) {
        (self.closure)(store, db)
    }
}

impl<State, Action, Event, Effect> Debug for DatabaseEffect<State, Action, Event, Effect> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DatabaseEffect(\"{}\")", self.debug)
    }
}

pub trait IsDatabaseEffect<State, Action, Event, Effect> {
    fn database_effect(&self) -> Option<&DatabaseEffect<State, Action, Event, Effect>>;
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
