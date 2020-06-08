use serde::Serialize;
use super::{CosterState, middleware::db::{IsDatabaseEffect, DatabaseEffect}, CosterAction, CosterEvent};

#[derive(Clone, Debug, Serialize)]
pub enum CosterEffect {
    Database(DatabaseEffect<CosterState, CosterAction, CosterEvent, CosterEffect>),
}

impl From<DatabaseEffect<CosterState, CosterAction, CosterEvent, CosterEffect>> for CosterEffect {
    fn from(effect: DatabaseEffect<CosterState, CosterAction, CosterEvent, CosterEffect>) -> Self {
        CosterEffect::Database(effect)
    }
}

impl IsDatabaseEffect<CosterState, CosterAction, CosterEvent, CosterEffect> for CosterEffect {
    fn database_effect(
        &self,
    ) -> Option<&DatabaseEffect<CosterState, CosterAction, CosterEvent, CosterEffect>> {
        match self {
            CosterEffect::Database(effect) => Some(effect),
        }
    }
}
