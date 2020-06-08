use serde::{Serialize, Deserialize};
use serde_diff::SerdeDiff;
use yew_state::StoreRef;
use super::{CosterEvent, CosterAction, CosterEffect, RouteType, AppRoute, middleware::{route::RouteState, localize::LocalizeState}};
use unic_langid::LanguageIdentifier;
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