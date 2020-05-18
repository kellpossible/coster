use super::Middleware;
use crate::StoreEvent;
use serde::Serialize;
use std::hash::Hash;

pub enum LogLevel {
    Trace,
    Debug,
    Warn,
    Info,
    Log,
}

/// Aiming to be something similar to https://github.com/LogRocket/redux-logger
pub struct WebLogger {
    _log_level: LogLevel,
}

impl<State, Action, Event> Middleware<State, Action, Event> for WebLogger
where
    State: Serialize,
    Action: Serialize,
    Event: StoreEvent + Clone + Hash + Eq,
{
    // fn invoke(
    //     &mut self,
    //     store: &mut crate::Store<State, Action, Error, Event>,
    //     action: Option<Action>,
    //     next: ReduceFn<State, Action, Error, Event>,
    // ) -> CallbackResults<Error> {
    //     let prev_state_js = JsValue::from_serde(&(**store.state()));

    //     // TODO: what will happen when action is None?
    //     let action_js = JsValue::from_serde(&action);

    //     let result = next(store, action);

    //     let next_state_js = JsValue::from_serde(&(**store.state()));

    //     result
    // }
}
