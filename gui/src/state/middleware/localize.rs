use i18n_embed::LanguageRequester;
use log::debug;
use std::{cell::RefCell, hash::Hash, rc::Rc};
use unic_langid::LanguageIdentifier;
use yew::{Component, ComponentLink};
use yew_state::{middleware::Middleware, Callback, Store, StoreEvent};

pub struct LocalizeMiddleware<LR> {
    pub language_requester: Rc<RefCell<LR>>,
}

impl<'a, LR> LocalizeMiddleware<LR>
where
    LR: LanguageRequester<'a>,
{
    pub fn new(language_requester: Rc<RefCell<LR>>) -> Self {
        Self { language_requester }
    }
}

impl<'a, LR, State, Action, Event> Middleware<State, Action, Event> for LocalizeMiddleware<LR>
where
    LR: LanguageRequester<'a>,
    Action: LocalizeAction,
{
    fn on_reduce(
        &self,
        store: &Store<State, Action, Event>,
        action: Option<Action>,
        reduce: yew_state::middleware::ReduceFn<State, Action, Event>,
    ) -> Vec<Event> {
        if let Some(action) = &action {
            if let Some(selected_language) = action.get_change_selected_language() {
                debug!(
                    "LocalizeMiddleware::on_reduce Processing selected language: {:?}",
                    selected_language
                );
                self.language_requester
                    .borrow_mut()
                    .set_language_override(selected_language.map(|l| l.clone()))
                    .unwrap();

                self.language_requester.borrow_mut().poll().unwrap();
            }
        }
        reduce(store, action)
    }
}

pub trait LocalizeEvent {
    fn language_changed() -> Self;
}

pub trait LocalizeAction {
    fn change_selected_language(selected_language: Option<LanguageIdentifier>) -> Self;
    fn get_change_selected_language(&self) -> Option<Option<&LanguageIdentifier>>;
}

pub trait LocalizeState {
    fn get_selected_language(&self) -> &Option<LanguageIdentifier>;
}

pub trait LocalizeStore<State, Event> {
    fn change_selected_language(&self, selected_language: Option<LanguageIdentifier>);
    fn subscribe_language_changed<COMP: Component>(
        &self,
        link: &ComponentLink<COMP>,
        message: COMP::Message,
    ) -> Callback<State, Event>
    where
        COMP::Message: Clone;
}

impl<State, Action, Event> LocalizeStore<State, Event> for Store<State, Action, Event>
where
    Action: LocalizeAction,
    State: LocalizeState + 'static,
    Event: LocalizeEvent + PartialEq + StoreEvent + Clone + Hash + Eq + 'static,
{
    fn change_selected_language(&self, selected_language: Option<LanguageIdentifier>) {
        self.dispatch(Action::change_selected_language(selected_language))
    }

    fn subscribe_language_changed<COMP: Component>(
        &self,
        link: &ComponentLink<COMP>,
        message: COMP::Message,
    ) -> Callback<State, Event>
    where
        COMP::Message: Clone,
    {
        let callback = link.callback(move |()| message.clone()).into();
        self.subscribe_event(&callback, LocalizeEvent::language_changed());
        callback
    }
}
