pub mod simple_logger;
pub mod web_logger;

use crate::Store;

pub type ReduceFn<State, Action, Event> =
    fn(&mut Store<State, Action, Event>, Option<Action>) -> Vec<Event>;

pub type NotifyFn<State, Action, Event> =
    fn(&mut Store<State, Action, Event>, Vec<Event>) -> Vec<Event>;

pub trait Middleware<State, Action, Event> {
    fn on_reduce(
        &self,
        store: &mut Store<State, Action, Event>,
        action: Option<Action>,
        reduce: ReduceFn<State, Action, Event>,
    ) -> Vec<Event> {
        reduce(store, action)
    }

    fn on_notify(
        &self,
        store: &mut Store<State, Action, Event>,
        events: Vec<Event>,
        notify: NotifyFn<State, Action, Event>,
    ) -> Vec<Event> {
        notify(store, events)
    }
}
