#![recursion_limit = "512"]

mod bulma;
mod components;
mod routing;
mod validation;
mod state;

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
use std::cell::RefCell;
use std::{fmt::Debug, rc::Rc};
use tr::tr;
use unic_langid::LanguageIdentifier;
use wasm_bindgen::prelude::*;
use yew::virtual_dom::VNode;
use yew::{html, Component, ComponentLink, Html, ShouldRender, services::{storage, StorageService}};
use yew_router::Switch;
use redux_rs::{Subscription, Store};
use state::{CosterState, StateStore};

#[derive(RustEmbed, I18nEmbed)]
#[folder = "i18n/mo"]
struct Translations;

language_loader!(WebLanguageLoader);

lazy_static! {
    static ref LANGUAGE_LOADER: WebLanguageLoader = WebLanguageLoader::new();
}

static TRANSLATIONS: Translations = Translations {};

#[derive(Switch, Clone)]
pub enum AppRoute {
    /// Matches the `/tab` route.
    #[to = "/tab"]
    CostingTab,
    /// Matches the `/new` route.
    #[to = "/new"]
    NewCostingTab,
    /// Matches the `/help` route.
    #[to = "/help"]
    Help,
    /// Matches the `/about` route.
    #[to = "/about"]
    About,
    /// Matches the `/` route.
    #[to = "/"]
    Index, // Order is important here, the index needs to be last.
}

impl Debug for AppRoute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Route (\"{}\")", self.to_string())
    }
}

pub type AppRouterRef = Rc<RefCell<SwitchRouteService<AppRoute>>>;
pub type LocalizerRef = Rc<Box<dyn Localizer<'static>>>;
pub type LanguageRequesterRef = Rc<RefCell<dyn LanguageRequester<'static>>>;

impl SwitchRoute for AppRoute {
    fn to_string(&self) -> String {
        match self {
            AppRoute::CostingTab => "/tab".to_string(),
            AppRoute::NewCostingTab => "/new".to_string(),
            AppRoute::Help => "/help".to_string(),
            AppRoute::About => "/about".to_string(),
            AppRoute::Index => "/".to_string(),
        }
    }
}

pub enum Msg {
    RouteChanged(Option<AppRoute>),
    ChangeRoute(AppRoute),
    LanguageChanged(LanguageIdentifier),
    StateChanged,
}

pub struct Model {
    language_requester: LanguageRequesterRef,
    localizer: LocalizerRef,
    router: AppRouterRef,
    route: Option<AppRoute>,
    link: ComponentLink<Self>,
    state_store: StateStore,
    storage: Option<StorageService>,
}

impl Model {
    fn page(&self, inner: Html) -> Html {
        let language_change_callback = self
            .link
            .callback(|selection| Msg::LanguageChanged(selection));

        html! {
            <Page
                router = self.router.clone()
                localizer = self.localizer.clone()
                language_requester = self.language_requester.clone()
                on_language_change = language_change_callback>
                { inner }
            </Page>
        }
    }
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let mut state_store = Store::new(state::reducer, state::CosterState::default());

        let state_change_callback = link.callback(|state: &CosterState| Msg::StateChanged);

        let state_change_listener: Subscription<CosterState> = move |state: &CosterState| {
            state_change_callback.emit(state);
        };
        state_store.subscribe(state_change_listener);

        let mut language_requester: WebLanguageRequester<'static> = WebLanguageRequester::new();

        let localizer = DefaultLocalizer::new(&*LANGUAGE_LOADER, &TRANSLATIONS);

        let localizer_rc: Rc<Box<dyn Localizer<'static>>> = Rc::new(Box::from(localizer));
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
            let selected_language_result: Result<String, anyhow::Error> = storage.restore("user-selected-language");

            match selected_language_result {
                Ok(selected_language_id) => {
                    let selected_language: unic_langid::LanguageIdentifier = selected_language_id.parse().unwrap();
                    debug!("Model::update restoring user-selected-language: {}", selected_language.to_string());
                    language_requester_rc.borrow_mut().set_languge_override(Some(selected_language)).unwrap();
                },
                Err(error) => {
                    error!("{}", error);
                }
            }
        }

        // Manually check the currently requested system language,
        // and update the listeners. When the system language changes,
        // this will automatically be triggered.
        language_requester_rc.borrow_mut().poll().unwrap();

        Model {
            link,
            language_requester: language_requester_rc,
            router: route_service_rc,
            route,
            localizer: localizer_rc,
            state_store: Rc::new(RefCell::new(state_store)),
            storage,
        }
    }

    fn update(&mut self, msg: Msg) -> ShouldRender {
        match msg {
            Msg::RouteChanged(route) => {
                debug!("Route changed: {:?}", route);
                self.route = route
            }
            Msg::ChangeRoute(route) => {
                self.router.borrow_mut().set_route(route);
            }
            Msg::LanguageChanged(lang) => {
                if let Some(storage) = &mut self.storage {
                    debug!("Model::update storing user-selected-language: {}", lang.to_string());
                    storage.store("user-selected-language", Ok(lang.to_string()));
                }
                debug!("Language changed in coster::lib {:?}", lang);
            }
        }
        true
    }

    fn view(&self) -> Html {
        debug!("Rendering coster::lib");

        let current_language = self.localizer.language_loader().current_language();

        let route_match_node = match &self.route {
            Some(AppRoute::CostingTab) => {
                debug!(target: "gui::router", "Detected CostingTab Route: {:?}", self.route);
                self.page(centered(html! {<CostingTab lang=current_language/>}))
            }
            Some(AppRoute::NewCostingTab) => {
                debug!(target: "gui::router", "Detected NewCostingTab Route: {:?}", self.route);
                self.page(centered(
                    html! {<NewCostingTab lang=current_language router=self.router.clone()/>},
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
                        html! {<CostingTabList router=self.router.clone() lang=current_language/>},
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
