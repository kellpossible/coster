mod middleware;
mod store;

pub use middleware::*;
pub use store::Store;

use std::rc::{Rc, Weak};

pub type CallbackResult<Error> = Result<(), Error>;
pub type CallbackResults<Error> = Result<(), Vec<Error>>;
pub type Listener<State, Error> = Weak<dyn Fn(Rc<State>) -> CallbackResult<Error>>;
pub type Callback<State, Error> = Rc<dyn Fn(Rc<State>) -> CallbackResult<Error>>;

pub trait Reducer<State, Action> {
    fn reduce(&self, state: &State, action: &Action) -> State;
}

impl<State, Action> Reducer<State, Action> for dyn Fn(&State, &Action) -> State {
    fn reduce(&self, state: &State, action: &Action) -> State {
        self(state, action)
    }
}
