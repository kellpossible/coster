pub mod simple_logger;
pub mod web_logger;

use crate::{CallbackResults, Store};

pub type ReduceFn<State, Action, Error, Event> =
    fn(&mut Store<State, Action, Error, Event>, Option<Action>) -> Vec<Event>;

pub type NotifyFn<State, Action, Error, Event> =
    fn(&mut Store<State, Action, Error, Event>, Vec<Event>) -> CallbackResults<Error>;

pub trait Middleware<State, Action, Error, Event> {
    fn on_reduce(
        &mut self,
        store: &mut Store<State, Action, Error, Event>,
        action: Option<Action>,
        reduce: ReduceFn<State, Action, Error, Event>,
    ) -> Vec<Event> {
        reduce(store, action)
    }

    fn on_notify(
        &mut self,
        store: &mut Store<State, Action, Error, Event>,
        action: Action,
        events: Vec<Event>,
        notify: NotifyFn<State, Action, Error, Event>,
    ) -> CallbackResults<Error> {
        notify(store, events)
    }
}
