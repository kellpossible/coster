use crate::ActionMiddleware;
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
    log_level: LogLevel
}

impl <State, Action, Error> ActionMiddleware<State, Action, Error> for WebLogger 
where
    State: Serialize,
    Action: Serialize, {
    fn invoke(
        &mut self,
        store: &mut crate::Store<State, Action, Error>,
        action: Option<Action>,
    ) -> Option<Action> {
        
        
        let prev_state_js = JsValue::from_serde(store.state());

        // TODO: what will happen when action is None?
        let action_js = JsValue::from_serde(&action); 

        // TODO: how do I get next state? Might need to change Middleware api. :'(
        action
    }
}