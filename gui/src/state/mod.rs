use std::{cell::RefCell, rc::Rc};
use unic_langid::LanguageIdentifier;
use yew_state::{Reducer, Store, StoreEvent};

pub type StateStore = Rc<RefCell<Store<CosterState, CosterAction, anyhow::Error, StateStoreEvent>>>;

pub struct CosterState {
    current_language: LanguageIdentifier,
}

impl Default for CosterState {
    fn default() -> Self {
        Self {
            current_language: "en".parse().unwrap(),
        }
    }
}

pub enum CosterAction {
    ChangeLanguage(LanguageIdentifier),
}

pub struct CosterReducer;

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum StateStoreEvent {
    LanguageChanged,
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
        action: &CosterAction,
    ) -> (CosterState, Vec<StateStoreEvent>) {
        let mut events = Vec::new();

        let state = match action {
            CosterAction::ChangeLanguage(language) => {
                events.push(StateStoreEvent::LanguageChanged);
                CosterState {
                    current_language: language.clone(),
                    ..*state
                }
            }
        };

        (state, events)
    }
}
