use crate::{CallbackResults, Store};

pub type NextFn<State, Action, Error, Event> =
    fn(&mut Store<State, Action, Error, Event>, Option<Action>) -> CallbackResults<Error>;

pub trait ActionMiddleware<State, Action, Error, Event> {
    fn invoke(
        &mut self,
        store: &mut Store<State, Action, Error, Event>,
        action: Option<Action>,
        next: NextFn<State, Action, Error, Event>,
    ) -> CallbackResults<Error>;
}
