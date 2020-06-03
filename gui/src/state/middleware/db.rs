use kvdb::KeyValueDB;
use yew_state::middleware::Middleware;

struct DatabaseMiddleware<DB> {
    database: DB,
}

impl<DB> DatabaseMiddleware<DB> where DB: KeyValueDB {
    pub fn new(database: DB) -> Self {
        Self {
            database
        }
    }
}

trait DataAction {
    
}

impl<DB, State, Action, Event> Middleware<State, Action, Event> for DatabaseMiddleware<DB> {
    fn on_reduce(
        &self,
        store: &yew_state::Store<State, Action, Event>,
        action: Option<Action>,
        reduce: yew_state::middleware::ReduceFn<State, Action, Event>,
    ) -> Vec<Event> {
        reduce(store, action)
    }

    fn on_notify(
        &self,
        store: &yew_state::Store<State, Action, Event>,
        events: Vec<Event>,
        notify: yew_state::middleware::NotifyFn<State, Action, Event>,
    ) -> Vec<Event> {
        notify(store, events)
    }
}
