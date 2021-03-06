use super::{
    middleware::localize::LocalizeEvent,
    RouteType,
};
use switch_router_middleware::RouteEvent;
use costing::TabID;
use serde::Serialize;
use reactive_state::StoreEvent;

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize)]
pub enum CosterEvent {
    StateChanged,
    LanguageChanged,
    RouteChanged,
    LastSelectedCurrencyChanged,
    TabsChanged,
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
