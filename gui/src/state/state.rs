use super::{
    middleware::localize::LocalizeState,
    AppRoute, CosterAction, CosterEffect, CosterEvent, RouteType,
};
use switch_router_middleware::RouteState;
use commodity::CommodityType;
use costing::Tab;
use serde::Serialize;
use std::rc::Rc;
use unic_langid::LanguageIdentifier;
use reactive_state::StoreRef;

pub type StateCallback = reactive_state::Callback<CosterState, CosterEvent>;

pub type StateStoreRef = StoreRef<CosterState, CosterAction, CosterEvent, CosterEffect>;

#[derive(Debug, Clone, Serialize)]
pub struct CosterState {
    pub selected_language: Option<LanguageIdentifier>,
    pub route: RouteType,
    pub last_selected_currency: Option<CommodityType>,
    pub tabs: Vec<Rc<Tab>>,
}

impl Default for CosterState {
    fn default() -> Self {
        Self {
            selected_language: None,
            route: RouteType::Valid(AppRoute::Index),
            last_selected_currency: None,
            tabs: Vec::new(),
        }
    }
}

// TODO: refactor this code into a macro
impl CosterState {
    pub fn change_route(&self, route: RouteType) -> Self {
        Self {
            selected_language: self.selected_language.clone(),
            route,
            last_selected_currency: self.last_selected_currency.clone(),
            tabs: self.tabs.clone(),
        }
    }

    pub fn change_selected_language(&self, selected_language: Option<LanguageIdentifier>) -> Self {
        Self {
            selected_language,
            route: self.route.clone(),
            last_selected_currency: self.last_selected_currency.clone(),
            tabs: self.tabs.clone(),
        }
    }

    pub fn change_last_selected_currency(
        &self,
        last_selected_currency: Option<CommodityType>,
    ) -> Self {
        Self {
            selected_language: self.selected_language.clone(),
            route: self.route.clone(),
            last_selected_currency,
            tabs: self.tabs.clone(),
        }
    }

    pub fn change_tabs(&self, tabs: Vec<Rc<Tab>>) -> Self {
        Self {
            selected_language: self.selected_language.clone(),
            route: self.route.clone(),
            last_selected_currency: self.last_selected_currency.clone(),
            tabs,
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
