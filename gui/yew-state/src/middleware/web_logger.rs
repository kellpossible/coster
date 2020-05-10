use super::{ActionMiddleware, NextFn};
use crate::CallbackResults;
use serde::Serialize;
use wasm_bindgen::JsValue;

pub enum LogLevel {
    Trace,
    Debug,
    Warn,
    Info,
    Log,
}

/// Aiming to be something similar to https://github.com/LogRocket/redux-logger
pub struct WebLogger {
    log_level: LogLevel,
}

impl<State, Action, Error> ActionMiddleware<State, Action, Error> for WebLogger
where
    State: Serialize,
    Action: Serialize,
{
    fn invoke(
        &mut self,
        store: &mut crate::Store<State, Action, Error>,
        action: Option<Action>,
        next: NextFn<State, Action, Error>,
    ) -> CallbackResults<Error> {
        let prev_state_js = JsValue::from_serde(store.state());

        // TODO: what will happen when action is None?
        let action_js = JsValue::from_serde(&action);

        let result = next(store, action);

        let next_state_js = JsValue::from_serde(store.state());

        result
    }
}
