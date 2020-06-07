pub mod middleware;
mod reducer;
mod route;

pub use reducer::*;
pub use route::*;

use middleware::{
    db::{IsDatabaseEffect, DatabaseEffect},
    localize::{LocalizeAction, LocalizeEvent, LocalizeState},
    route::{IsRouteAction, RouteAction, RouteEvent, RouteState},
};
use serde::{Deserialize, Serialize};
use serde_diff::SerdeDiff;
use std::{
    fmt::{Debug, Display},
};

use unic_langid::LanguageIdentifier;
use yew_state::{StoreEvent, StoreRef};

pub type StateCallback = yew_state::Callback<CosterState, CosterEvent>;

pub type StateStoreRef = StoreRef<CosterState, CosterAction, CosterEvent, CosterEffect>;

#[derive(Debug, Clone, SerdeDiff, Serialize, Deserialize)]
pub struct CosterState {
    #[serde_diff(opaque)]
    pub selected_language: Option<LanguageIdentifier>,
    #[serde_diff(opaque)]
    pub route: RouteType,
}

impl Default for CosterState {
    fn default() -> Self {
        Self {
            selected_language: None,
            route: RouteType::Valid(AppRoute::Index),
        }
    }
}

impl CosterState {
    pub fn change_route(&self, route: RouteType) -> Self {
        Self {
            selected_language: self.selected_language.clone(),
            route,
        }
    }

    pub fn change_selected_language(&self, selected_language: Option<LanguageIdentifier>) -> Self {
        Self {
            selected_language,
            route: self.route.clone(),
        }
    }
}

impl RouteState<RouteType> for CosterState {
    fn get_route(&self) -> &RouteType {
        &self.route
    }
}

impl LocalizeState for CosterState {
    fn get_selected_language(&self) -> &Option<LanguageIdentifier> {
        &self.selected_language
    }
}

#[derive(Debug, PartialEq, Clone, Serialize)]
pub enum CosterAction {
    ChangeSelectedLanguage(Option<LanguageIdentifier>),
    RouteAction(RouteAction<RouteType>),
}

impl Display for CosterAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CosterAction::ChangeSelectedLanguage(selected_language) => {
                let language_display = match selected_language {
                    Some(language) => language.to_string(),
                    None => "None".to_string(),
                };
                write!(f, "ChangeSelectedLanguage({})", language_display)
            }
            CosterAction::RouteAction(route_action) => write!(f, "RouteAction::{}", route_action),
        }
    }
}

impl LocalizeAction for CosterAction {
    fn change_selected_language(selected_language: Option<LanguageIdentifier>) -> Self {
        CosterAction::ChangeSelectedLanguage(selected_language)
    }
    fn get_change_selected_language(&self) -> Option<Option<&LanguageIdentifier>> {
        match self {
            CosterAction::ChangeSelectedLanguage(selected_language) => {
                Some(selected_language.as_ref())
            }
            _ => None,
        }
    }
}

impl From<RouteAction<RouteType>> for CosterAction {
    fn from(route_action: RouteAction<RouteType>) -> Self {
        CosterAction::RouteAction(route_action)
    }
}

impl IsRouteAction<RouteType> for CosterAction {
    fn route_action(&self) -> Option<&RouteAction<RouteType>> {
        match self {
            CosterAction::RouteAction(route_action) => Some(route_action),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize)]
pub enum CosterEvent {
    LanguageChanged,
    RouteChanged,
    None,
}

impl StoreEvent for CosterEvent {
    fn none() -> Self {
        CosterEvent::None
    }
    fn is_none(&self) -> bool {
        self == &Self::none()
    }
}

impl LocalizeEvent for CosterEvent {
    fn language_changed() -> Self {
        CosterEvent::LanguageChanged
    }
}

impl RouteEvent<RouteType> for CosterEvent {
    fn route_changed() -> Self {
        CosterEvent::RouteChanged
    }
}

#[derive(Clone)]
pub enum CosterEffect {
    Database(DatabaseEffect<CosterState, CosterAction, CosterEvent, CosterEffect>),
}

impl IsDatabaseEffect<CosterState, CosterAction, CosterEvent, CosterEffect> for CosterEffect {
    fn database_effect(&self) -> Option<&DatabaseEffect<CosterState, CosterAction, CosterEvent, CosterEffect>> {
        match self {
            CosterEffect::Database(effect) => {
                Some(effect)
            }
        }
    }
}