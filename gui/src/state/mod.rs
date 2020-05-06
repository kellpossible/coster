use unic_langid::LanguageIdentifier;
use std::{rc::Rc, cell::RefCell};
use redux_rs::Store;

pub type StateStore = Rc<RefCell<Store<State, Action>>>;

pub struct State {
    current_language: LanguageIdentifier
}

impl Default for State {
    fn default() -> Self {
        Self {
            current_language: "en".parse().unwrap()
        }
    }
}

pub enum Action {
    ChangeLanguage(LanguageIdentifier)
}

pub fn reducer(state: &State, action: &Action) -> State {
    match action {
        Action::ChangeLanguage(language_identifier) => {
            State {
                current_language: language_identifier.clone()
            }
        }
    }
}