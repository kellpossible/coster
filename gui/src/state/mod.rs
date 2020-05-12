use std::{cell::RefCell, rc::Rc};
use unic_langid::LanguageIdentifier;
use yew_state::{Reducer, Store};

pub type StateStore = Rc<RefCell<Store<CosterState, CosterAction, anyhow::Error, ()>>>;

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

impl Reducer<CosterState, CosterAction> for CosterReducer {
    fn reduce(&self, state: &CosterState, action: &CosterAction) -> CosterState {
        match action {
            CosterAction::ChangeLanguage(language) => CosterState {
                current_language: language.clone(),
                ..*state
            },
        }
    }
}
