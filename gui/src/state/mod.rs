use unic_langid::LanguageIdentifier;
use std::{rc::Rc, cell::RefCell};
use yew_state::{Reducer, Store};

pub type StateStore = Rc<RefCell<Store<CosterState, CosterAction, anyhow::Error>>>;

pub struct CustomData(i32);

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
    ChangeLanguage(LanguageIdentifier)
}

pub struct CosterReducer {

}

impl Reducer<CosterState, CosterAction> for CosterReducer {
    fn reduce(&self, state: &CosterState, action: &CosterAction) -> CosterState {
        match action {
            CosterAction::ChangeLanguage(language) => {
                CosterState {
                    current_language: *language,
                    ..*state
                }
            }
        }
    }
}