use crate::{
    middleware::{Middleware, ReduceFn},
    Store, StoreEvent,
};
use std::{fmt::Debug, hash::Hash};

pub enum LogLevel {
    Trace,
    Debug,
    Warn,
    Info,
}

impl LogLevel {
    pub fn log<S: AsRef<str>>(&self, message: S) {
        match self {
            LogLevel::Trace => log::trace!("{}", message.as_ref()),
            LogLevel::Debug => log::debug!("{}", message.as_ref()),
            LogLevel::Warn => log::warn!("{}", message.as_ref()),
            LogLevel::Info => log::info!("{}", message.as_ref()),
        }
    }
}

impl Default for LogLevel {
    fn default() -> Self {
        LogLevel::Debug
    }
}

pub struct SimpleLoggerMiddleware {
    log_level: LogLevel,
}

impl SimpleLoggerMiddleware {
    pub fn new() -> Self {
        SimpleLoggerMiddleware {
            log_level: LogLevel::default(),
        }
    }

    pub fn log_level(mut self, log_level: LogLevel) -> Self {
        self.log_level = log_level;
        self
    }
}

impl<State, Action, Event> Middleware<State, Action, Event> for SimpleLoggerMiddleware
where
    Event: StoreEvent + Clone + Hash + Eq + Debug,
    State: Debug,
    Action: Debug,
{
    fn on_reduce(
        &mut self,
        store: &mut Store<State, Action, Event>,
        action: Option<Action>,
        reduce: ReduceFn<State, Action, Event>,
    ) -> Vec<Event> {
        let was_action = match &action {
            Some(action) => {
                self.log_level
                    .log(format!("prev state: {:?}", store.state()));
                self.log_level.log(format!("action: {:?}", action));
                true
            }
            None => {
                self.log_level.log("action: None");
                false
            }
        };

        let events = reduce(store, action);

        if was_action {
            self.log_level
                .log(format!("next state: {:?}", store.state()));
        }

        events
    }

    fn on_notify(
        &mut self,
        store: &mut Store<State, Action, Event>,
        action: Action,
        events: Vec<Event>,
        notify: super::NotifyFn<State, Action, Event>,
    ) {
        self.log_level.log("on_notify");
        for event in &events {
            self.log_level.log(format!(
                "event {:?} dispatched due to action {:?}",
                event, action
            ));
        }

        notify(store, events);
    }
}
