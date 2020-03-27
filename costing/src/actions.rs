use crate::tab::Tab;
use crate::error::CostingError;
use chrono::{DateTime, Utc};
use std::fmt;
use std::rc::Weak;

/// Represents an action which can modify [ProgramState](ProgramState).
pub trait UserAction: fmt::Display + fmt::Debug {
    /// The date/time that the action was performed.
    fn datetime(&self) -> DateTime<Utc>;

    /// Perform the action to mutate the [Tab](Tab).
    fn perform(&self, tab: &mut Tab) -> Result<(), CostingError>;

    /// Retrieve the previous user action
    fn previous_action(&self) -> Option<Weak<dyn UserAction>>;
}