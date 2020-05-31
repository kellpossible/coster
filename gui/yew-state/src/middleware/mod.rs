//! [Middleware] used to modify the behaviour of a [Store] during a
//! [Store::dispatch()]. This module also contains some simple
//! middleware implementations which can be used as utilities in an
//! application.

pub mod simple_logger;
pub mod web_logger;

use crate::Store;

/// Executes subsequent middleware and then runs the [Reducer](crate::Reducer).
pub type ReduceFn<State, Action, Event> =
    fn(&Store<State, Action, Event>, Option<Action>) -> Vec<Event>;

/// Executes subsequent middleware and then notifies the listeners.
pub type NotifyFn<State, Action, Event> =
    fn(&Store<State, Action, Event>, Vec<Event>) -> Vec<Event>;

/// `Middleware` used to modify the behaviour of a [Store] during a
/// [Store::dispatch()].
pub trait Middleware<State, Action, Event> {
    /// This method is invoked by the [Store] during a
    /// [Store::dispatch()] just before the `Action` is sent to the
    /// [Reducer](crate::Reducer). It is necessary to call the
    /// provided `reduce` function, which executes subsequent
    /// middleware and runs the [Reducer](crate::Reducer), and usually
    /// the events produced by the `reduce` function are returned from
    /// this method.
    ///
    /// This method allows modifying the action in question, or even
    /// removing it, preventing the [Reducer](crate::Reducer) from
    /// processing the action. It also allows modifying the events
    /// produced by the [Reducer](crate::Reducer) before the
    /// [Middleware::on_notify()] is invoked and they are sent to the
    /// [Store] listeners.
    fn on_reduce(
        &self,
        store: &Store<State, Action, Event>,
        action: Option<Action>,
        reduce: ReduceFn<State, Action, Event>,
    ) -> Vec<Event> {
        reduce(store, action)
    }

    /// This method is invoked by the [Store] during a
    /// [Store::dispatch()] after the [Reducer](crate::Reducer) has
    /// processed the `Action` and all [Middleware::on_reduce()]
    /// methods have completed, just before resulting events are
    /// sent to the store listeners. It is necessary to call the
    /// provided `notify` function, which executes subsequent
    /// middleware and then notifies the listeners.
    ///
    /// This method allows modifying the events in question before the
    /// listeners are notified.
    fn on_notify(
        &self,
        store: &Store<State, Action, Event>,
        events: Vec<Event>,
        notify: NotifyFn<State, Action, Event>,
    ) -> Vec<Event> {
        notify(store, events)
    }
}
