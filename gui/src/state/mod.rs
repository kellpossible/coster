pub mod middleware;
mod reducer;
mod route;

pub use reducer::*;
pub use route::*;

use middleware::{
    db::{DatabaseEffect, IsDatabaseEffect},
    localize::{ChangeSelectedLanguage, LocalizeAction, LocalizeEvent, LocalizeState},
    route::{IsRouteAction, RouteAction, RouteEvent, RouteState},
};
use serde::{Deserialize, Serialize};
use serde_diff::SerdeDiff;
use std::fmt::{Debug, Display};

use unic_langid::LanguageIdentifier;
use yew_state::{StoreEvent, StoreRef};
use commodity::CommodityType;

pub type StateCallback = yew_state::Callback<CosterState, CosterEvent>;

pub type StateStoreRef = StoreRef<CosterState, CosterAction, CosterEvent, CosterEffect>;

#[derive(Debug, Clone, SerdeDiff, Serialize, Deserialize)]
pub struct CosterState {
    #[serde_diff(opaque)]
    pub selected_language: Option<LanguageIdentifier>,
    #[serde_diff(opaque)]
    pub route: RouteType,
    #[serde_diff(opaque)]
    pub last_selected_currency: Option<CommodityType>,
}

impl Default for CosterState {
    fn default() -> Self {
        Self {
            selected_language: None,
            route: RouteType::Valid(AppRoute::Index),
            last_selected_currency: None,
        }
    }
}

impl CosterState {
    pub fn change_route(&self, route: RouteType) -> Self {
        Self {
            selected_language: self.selected_language.clone(),
            route,
            last_selected_currency: self.last_selected_currency.clone(),
        }
    }

    pub fn change_selected_language(&self, selected_language: Option<LanguageIdentifier>) -> Self {
        Self {
            selected_language,
            route: self.route.clone(),
            last_selected_currency: self.last_selected_currency.clone(),
        }
    }

    pub fn change_last_selected_currency(&self, last_selected_currency: Option<CommodityType>) -> Self {
        Self {
            selected_language: self.selected_language.clone(),
            route: self.route.clone(),
            last_selected_currency,
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
    /// Selected language, and whether or not to write the value to the database.
    ChangeSelectedLanguage(ChangeSelectedLanguage),
    RouteAction(RouteAction<RouteType>),
    LoadDatabase,
    ChangeLastSelectedCurrency(Option<CommodityType>),
}

impl Display for CosterAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CosterAction::ChangeSelectedLanguage(action) => write!(f, "{}", action),
            CosterAction::RouteAction(route_action) => write!(f, "RouteAction::{}", route_action),
            CosterAction::LoadDatabase => write!(f, "LoadDatabase"),
            CosterAction::ChangeLastSelectedCurrency(currency) => {
                let currency_display = match currency {
                    Some(currency) => format!("{}", currency),
                    None => "None".to_string(),
                };
                write!(f, "ChangeLastSelectedCurrency({})", currency_display)
            }
        }
    }
}

impl LocalizeAction for CosterAction {
    fn change_selected_language(action: ChangeSelectedLanguage) -> Self {
        CosterAction::ChangeSelectedLanguage(action)
    }
    fn get_change_selected_language(&self) -> Option<&ChangeSelectedLanguage> {
        match self {
            CosterAction::ChangeSelectedLanguage(action) => Some(action),
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
    StateChanged,
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
