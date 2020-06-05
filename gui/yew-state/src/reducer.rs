use std::rc::Rc;

/// Using the [reduce()](Reducer::reduce()) method, implementors of
/// this trait take an `Action` submitted to a store via
/// [Store::dispatch()](crate::Store::dispatch()) and modifies the
/// `State` in the store, producing a new `State`, and also producing
/// events associated with the `Action` and state modifications that
/// occurred.
pub trait Reducer<State, Action, Event> {
    /// Take an `Action` submitted to a store via
    /// [Store::dispatch()](crate::Store::dispatch()) and modifies the
    /// `prev_state`, producing a new `State`, and also producing
    /// events associated with the `Action` and state modifications
    /// that occurred.
    ///
    /// `Events`s should genearlly be treated purely as a notification
    /// that some subset of the state has been modified, such that
    /// playing the events and state transitions in reverse will
    /// result in the same application behaviour.
    ///
    /// If no `Event`s are returned then it is assumed that the state
    /// has not changed, and store listeners do not need to be
    /// notified.
    fn reduce(&self, prev_state: Rc<State>, action: Action) -> ReducerResult<State, Event>;
}

impl<State, Action, Event> Reducer<State, Action, Event>
    for dyn Fn(Rc<State>, Action) -> (Rc<State>, Vec<Event>)
{
    fn reduce(&self, prev_state: Rc<State>, action: Action) -> ReducerResult<State, Event> {
        let (state, events) = self(prev_state, action);
        ReducerResult { state, events }
    }
}

/// The result of a [Reducer::reduce()] function.
///
/// `Events`s should genearlly be treated purely as a notification
/// that some subset of the state has been modified, such that
/// playing the events and state transitions in reverse will
/// result in the same application behaviour.
pub struct ReducerResult<State, Event> {
    pub state: Rc<State>,
    pub events: Vec<Event>,
}
