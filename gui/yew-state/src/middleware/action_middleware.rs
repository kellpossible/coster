use crate::{CallbackResults, Store};

pub type NextFn<State, Action, Error> =
    fn(&mut Store<State, Action, Error>, Option<Action>) -> CallbackResults<Error>;

pub trait ActionMiddleware<State, Action, Error> {
    fn invoke(
        &mut self,
        store: &mut Store<State, Action, Error>,
        action: Option<Action>,
        next: NextFn<State, Action, Error>,
    ) -> CallbackResults<Error>;
}
