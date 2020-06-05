use serde::Serialize;
use std::{fmt::Debug, hash::Hash, rc::Rc};
use yew_state::Store;

#[derive(Clone, Serialize)]
pub struct DatabaseDispatch<DB, State, Action, Event> {
    #[serde(skip)]
    closure: Rc<dyn Fn(&Store<State, Action, Event>, &DB)>,
    ignore_during_read: bool,
}

impl<DB, State, Action, Event> PartialEq for DatabaseDispatch<DB, State, Action, Event> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.closure, &other.closure)
            && self.ignore_during_read == other.ignore_during_read
    }
}

impl<DB, State, Action, Event> Eq for DatabaseDispatch<DB, State, Action, Event> {}

impl<DB, State, Action, Event> Hash for DatabaseDispatch<DB, State, Action, Event> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let pointer = self.closure.as_ref() as *const (dyn Fn(&Store<State, Action, Event>, &DB))
            as *const ();
        state.write_usize(pointer as usize);
        self.ignore_during_read.hash(state);
    }
}

impl<DB, State, Action, Event> Debug for DatabaseDispatch<DB, State, Action, Event> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "DatabaseDispatch(closure @ {:p}, ignore_during_read: {:?}",
            self.closure, self.ignore_during_read
        )
    }
}

impl<DB, State, Action, Event> DatabaseDispatch<DB, State, Action, Event> {
    pub fn run(&self, store: &Store<State, Action, Event>, database: &DB, reading_database: bool) {
        if !(self.ignore_during_read && reading_database) {
            (self.closure)(store, database)
        }
    }
}

impl<F, DB, State, Action, Event> From<F> for DatabaseDispatch<DB, State, Action, Event>
where
    F: Fn(&Store<State, Action, Event>, &DB) + 'static,
{
    fn from(f: F) -> Self {
        DatabaseDispatch {
            closure: Rc::new(f),
            ignore_during_read: true,
        }
    }
}
