mod route;

pub use route::*;

use unic_langid::LanguageIdentifier;
use yew_state::{Reducer, StoreEvent, StoreRef};

pub type StateStoreRef = StoreRef<CosterState, CosterAction, StateStoreEvent>;

pub struct CosterState {
    language: LanguageIdentifier,
    route: AppRoute,
}

impl Default for CosterState {
    fn default() -> Self {
        Self {
            language: "en".parse().unwrap(),
            route: AppRoute::Index,
        }
    }
}

impl CosterState {
    pub fn change_route(&self, route: AppRoute) -> Self {
        Self {
            language: self.language.clone(),
            route,
        }
    }

    pub fn change_language(&self, language: LanguageIdentifier) -> Self {
        Self {
            language,
            route: self.route.clone(),
        }
    }
}

pub enum CosterAction {
    ChangeLanguage(LanguageIdentifier),
    ChangeRoute(AppRoute),
}

pub struct CosterReducer;

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum StateStoreEvent {
    LanguageChanged,
    RouteChanged(AppRoute),
    None,
}

impl StoreEvent for StateStoreEvent {
    fn none() -> Self {
        StateStoreEvent::None
    }
    fn is_none(&self) -> bool {
        self == &Self::none()
    }
}

impl Reducer<CosterState, CosterAction, StateStoreEvent> for CosterReducer {
    fn reduce(
        &self,
        state: &CosterState,
        action: CosterAction,
    ) -> (CosterState, Vec<StateStoreEvent>) {
        let mut events = Vec::new();

        let state = match action {
            CosterAction::ChangeLanguage(language) => {
                events.push(StateStoreEvent::LanguageChanged);
                state.change_language(language)
            }
            CosterAction::ChangeRoute(route) => {
                events.push(StateStoreEvent::RouteChanged(route.clone()));
                state.change_route(route)
            }
        };

        (state, events)
    }
}
