use crate::{Store, ReducerResult, CallbackResults};

pub type ReduceFn<State, Action, Error, Event> =
    fn(&mut Store<State, Action, Error, Event>, Option<Action>) -> ReducerResult<State, Event>;

pub type NotifyFn<State, Action, Error, Event> =
    fn(&mut Store<State, Action, Error, Event>, ReducerResult<State, Event>) -> CallbackResults<Error>;

pub trait ActionMiddleware<State, Action, Error, Event> {
    fn on_reduce(
        &mut self,
        store: &mut Store<State, Action, Error, Event>,
        action: Option<Action>,
        reduce: ReduceFn<State, Action, Error, Event>,
    ) -> ReducerResult<State, Event> {
        reduce(store, action)
    }

    fn on_notify(
        &mut self,
        store: &mut Store<State, Action, Error, Event>,
        action: Action,
        reducer_result: ReducerResult<State, Event>,
        notify: NotifyFn<State, Action, Error, Event>,
    ) -> CallbackResults<Error> {
        notify(store, reducer_result)
    }
}
