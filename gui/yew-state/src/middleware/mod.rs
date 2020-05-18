pub mod simple_logger;
pub mod web_logger;

use crate::Store;

pub type ReduceFn<State, Action, Event> =
    fn(&mut Store<State, Action, Event>, Option<Action>) -> Vec<Event>;

pub type NotifyFn<State, Action, Event> = fn(&mut Store<State, Action, Event>, Vec<Event>);

pub trait Middleware<State, Action, Event> {
    fn on_reduce(
        &mut self,
        store: &mut Store<State, Action, Event>,
        action: Option<Action>,
        reduce: ReduceFn<State, Action, Event>,
    ) -> Vec<Event> {
        reduce(store, action)
    }

    fn on_notify(
        &mut self,
        store: &mut Store<State, Action, Event>,
        action: Action,
        events: Vec<Event>,
        notify: NotifyFn<State, Action, Event>,
    ) {
        notify(store, events);
    }
}
