use serde::Serialize;
use std::fmt::Debug;
use yew_state::Store;

#[derive(Serialize)]
pub struct DatabaseDispatch<DB, State, Action, Event, Effect> {
    #[serde(skip)]
    closure: Box<dyn Fn(&Store<State, Action, Event, Effect>, &DB)>,
    ignore_during_read: bool,
}

impl<DB, State, Action, Event, Effect> PartialEq
    for DatabaseDispatch<DB, State, Action, Event, Effect>
{
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(&self.closure, &other.closure)
            && self.ignore_during_read == other.ignore_during_read
    }
}

impl<DB, State, Action, Event, Effect> Eq for DatabaseDispatch<DB, State, Action, Event, Effect> {}

impl<DB, State, Action, Event, Effect> Debug
    for DatabaseDispatch<DB, State, Action, Event, Effect>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "DatabaseDispatch(closure @ {:p}, ignore_during_read: {:?}",
            self.closure, self.ignore_during_read
        )
    }
}

impl<DB, State, Action, Event, Effect> DatabaseDispatch<DB, State, Action, Event, Effect> {
    pub fn run(
        &self,
        store: &Store<State, Action, Event, Effect>,
        database: &DB,
        reading_database: bool,
    ) {
        if !(self.ignore_during_read && reading_database) {
            (self.closure)(store, database)
        }
    }
}

impl<F, DB, State, Action, Event, Effect> From<F>
    for DatabaseDispatch<DB, State, Action, Event, Effect>
where
    F: Fn(&Store<State, Action, Event, Effect>, &DB) + 'static,
{
    fn from(f: F) -> Self {
        DatabaseDispatch {
            closure: Box::new(f),
            ignore_during_read: true,
        }
    }
}
