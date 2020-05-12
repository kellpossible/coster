use crate::{ActionMiddleware, CallbackResults, NextFn, Store};
use std::fmt::{Debug, Display};

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

pub struct SimpleLogger {
    log_level: LogLevel,
}

impl SimpleLogger {
    pub fn new() -> Self {
        SimpleLogger {
            log_level: LogLevel::default(),
        }
    }

    pub fn log_level(mut self, log_level: LogLevel) -> Self {
        self.log_level = log_level;
        self
    }
}

impl<State, Action, Error> ActionMiddleware<State, Action, Error> for SimpleLogger
where
    State: Debug,
    Action: Debug,
    Error: Display,
{
    fn invoke(
        &mut self,
        store: &mut Store<State, Action, Error>,
        action: Option<Action>,
        next: NextFn<State, Action, Error>,
    ) -> CallbackResults<Error> {
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

        let result = next(store, action);

        if was_action {
            self.log_level
                .log(format!("next state: {:?}", store.state()));
        }

        if let Err(errors) = &result {
            let mut message = format!("{} listener errors:\n", errors.len());

            for error in errors {
                message.push_str(&format!("{}\n", error));
            }

            self.log_level.log(message);
        }

        result
    }
}
