#![recursion_limit = "512"]

mod bulma;
mod components;
mod routing;
mod state;
mod validation;

use components::costing_tab::CostingTab;
use components::costing_tab_list::CostingTabList;
use components::new_costing_tab::NewCostingTab;
use components::pages::{centered, Page};
use routing::{SwitchRoute, SwitchRouteService};

use i18n_embed::{
    language_loader, DefaultLocalizer, I18nEmbed, LanguageRequester, Localizer,
    WebLanguageRequester,
};
use lazy_static::lazy_static;
use log;
use log::{debug, error};
use rust_embed::RustEmbed;
use state::{AppRoute, CosterReducer, CosterState, StateStoreEvent, RouteMiddleware, StateStoreRef, CosterAction};
use std::cell::RefCell;
use std::{fmt::Debug, rc::Rc};
use tr::tr;
use unic_langid::LanguageIdentifier;
use wasm_bindgen::prelude::*;
use yew::virtual_dom::VNode;
use yew::{
    html,
    services::{storage, StorageService},
    Component, ComponentLink, Html, ShouldRender,
};

#[derive(RustEmbed, I18nEmbed)]
#[folder = "i18n/mo"]
struct Translations;

language_loader!(WebLanguageLoader);

lazy_static! {
    static ref LANGUAGE_LOADER: WebLanguageLoader = WebLanguageLoader::new();
}

static TRANSLATIONS: Translations = Translations {};

impl Debug for AppRoute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Route (\"{}\")", self.to_string())
    }
}

pub type AppRouterRef = Rc<RefCell<SwitchRouteService<AppRoute>>>;
pub type LocalizerRef = Rc<dyn Localizer<'static>>;
pub type LanguageRequesterRef = Rc<RefCell<dyn LanguageRequester<'static>>>;

pub enum Msg {
    RouteChanged(Option<AppRoute>),
    LanguageChanged(LanguageIdentifier),
    StateChanged(Rc<CosterState>, StateStoreEvent),
}

pub struct Model {
    language_requester: LanguageRequesterRef,
    localizer: LocalizerRef,
    route: Option<AppRoute>,
    link: ComponentLink<Self>,
    state_store: StateStoreRef,
    storage: Option<StorageService>,
    _state_callback: yew_state::Callback<CosterState, anyhow::Error, StateStoreEvent>,
}

impl Model {
    fn page(&self, inner: Html) -> Html {
        let language_change_callback = self
            .link
            .callback(|selection| Msg::LanguageChanged(selection));

        html! {
            <Page
                state_store = self.state_store.clone()
                localizer = self.localizer.clone()
                language_requester = self.language_requester.clone()>
                { inner }
            </Page>
        }
    }
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let state_store: StateStoreRef = StateStoreRef::new(CosterReducer, CosterState::default());
        let route_middleware = RouteMiddleware::new(&state_store);
        state_store.add_middleware(route_middleware);

        let mut language_requester: WebLanguageRequester<'static> = WebLanguageRequester::new();

        let localizer = DefaultLocalizer::new(&*LANGUAGE_LOADER, &TRANSLATIONS);

        let localizer_rc: Rc<dyn Localizer<'static>> = Rc::new(localizer);
        language_requester.add_listener(&localizer_rc);

        let language_requester_rc: Rc<RefCell<dyn LanguageRequester<'static>>> =
            Rc::new(RefCell::from(language_requester));

        let mut route_service: SwitchRouteService<AppRoute> = SwitchRouteService::new();
        let route_callback = link.callback(Msg::RouteChanged);
        route_service.register_callback(route_callback);
        let route = route_service.get_route();
        let route_service_rc = Rc::new(RefCell::from(route_service));

        debug!(target: "gui::router", "Initial Route: {:?}", route);

        let storage = StorageService::new(storage::Area::Local).ok();

        if let Some(storage) = &storage {
            let selected_language_result: Result<String, anyhow::Error> =
                storage.restore("user-selected-language");

            match selected_language_result {
                Ok(selected_language_id) => {
                    let selected_language: unic_langid::LanguageIdentifier =
                        selected_language_id.parse().unwrap();
                    debug!(
                        "Model::update restoring user-selected-language: {}",
                        selected_language.to_string()
                    );
                    language_requester_rc
                        .borrow_mut()
                        .set_languge_override(Some(selected_language))
                        .unwrap();
                }
                Err(error) => {
                    error!("{}", error);
                }
            }
        }

        // Manually check the currently requested system language,
        // and update the listeners. When the system language changes,
        // this will automatically be triggered.
        language_requester_rc.borrow_mut().poll().unwrap();

        let state_callback: yew_state::Callback<CosterState, anyhow::Error, StateStoreEvent> = link
            .callback(|(state, event)| Msg::StateChanged(state, event))
            .into();
        state_store.subscribe(&state_callback);

        Model {
            link,
            language_requester: language_requester_rc,
            route,
            localizer: localizer_rc,
            state_store,
            storage,
            _state_callback: state_callback,
        }
    }

    fn update(&mut self, msg: Msg) -> ShouldRender {
        match msg {
            Msg::RouteChanged(route) => {
                debug!("Route changed: {:?}", route);
                self.route = route;
                true
            }
            Msg::LanguageChanged(lang) => {
                if let Some(storage) = &mut self.storage {
                    debug!(
                        "Model::update storing user-selected-language: {}",
                        lang.to_string()
                    );
                    storage.store("user-selected-language", Ok(lang.to_string()));
                }
                debug!("Language changed in coster::lib {:?}", lang);
                true
            }
            Msg::StateChanged(state, event) => false,
        }
    }

    fn view(&self) -> Html {
        debug!("Rendering coster::lib");

        let current_language = self.localizer.language_loader().current_language();

        let route_match_node = match &self.route {
            Some(AppRoute::CostingTab) => {
                debug!(target: "gui::router", "Detected CostingTab Route: {:?}", self.route);
                self.page(centered(
                    html! {<CostingTab state_store=self.state_store.clone()/>},
                ))
            }
            Some(AppRoute::NewCostingTab) => {
                debug!(target: "gui::router", "Detected NewCostingTab Route: {:?}", self.route);
                self.page(centered(
                    html! {<NewCostingTab state_store=self.state_store.clone()/>},
                ))
            }
            Some(AppRoute::Help) => {
                self.page(html! { <h1 class="title is-1">{ tr!("Help for Coster") }</h1> })
            }
            Some(AppRoute::About) => {
                self.page(html! { <h1 class="title is-1">{ tr!("About Coster") }</h1> })
            }
            Some(AppRoute::Index) => {
                if self.route.as_ref().unwrap().to_string() == "/" {
                    debug!(target: "gui::router", "Detected CostingTabListPage Route: {:?}", self.route);
                    self.page(centered(
                        html! {<CostingTabList state_store=self.state_store.clone()/>},
                    ))
                } else {
                    debug!(target: "gui::router", "Detected Invalid Route: {:?}", self.route);
                    VNode::from("404")
                }
            }
            _ => {
                debug!(target: "gui::router", "Detected Invalid Route: {:?}", self.route);
                VNode::from("404")
            }
        };

        html! {
            <>
            {
                route_match_node
            }
            </>
        }
    }

    fn change(&mut self, _: Self::Properties) -> ShouldRender {
        false
    }
}

// Called when the wasm module is instantiated
#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    #[cfg(feature = "logging")]
    wasm_logger::init(wasm_logger::Config::default());

    yew::start_app::<Model>();
    Ok(())
}
