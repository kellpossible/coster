#![recursion_limit = "512"]

mod bulma;
mod components;
mod state;
mod validation;

use components::costing_tab::CostingTab;
use components::costing_tab_list::CostingTabList;
use components::new_costing_tab::NewCostingTab;
use components::pages::{centered, Page};
use switch_router::{SwitchRoute, SwitchRouteService};

use i18n_embed::{
    language_loader, DefaultLocalizer, I18nEmbed, LanguageRequester, Localizer,
    WebLanguageRequester,
};
use lazy_static::lazy_static;
use log;
use log::{debug, error};
use rust_embed::RustEmbed;
use state::{
    middleware::{
        db::DatabaseMiddleware,
        localize::LocalizeMiddleware,
        route::{RouteAction, RouteMiddleware},
    },
    AppRoute, CosterAction, CosterEvent, CosterReducer, CosterState, RouteType, StateStoreRef,
};
use std::cell::RefCell;
use std::rc::Rc;
use tr::tr;
use wasm_bindgen::prelude::*;
use yew::virtual_dom::VNode;
use yew::{
    html,
    services::{storage, StorageService},
    Component, ComponentLink, Html, ShouldRender,
};
use yew_state::middleware::web_logger::{LogLevel, WebLoggerMiddleware};

#[derive(RustEmbed, I18nEmbed)]
#[folder = "i18n/mo"]
struct Translations;

language_loader!(WebLanguageLoader);

lazy_static! {
    static ref LANGUAGE_LOADER: WebLanguageLoader = WebLanguageLoader::new();
}

static TRANSLATIONS: Translations = Translations;

pub type AppRouterRef = Rc<RefCell<SwitchRouteService<AppRoute>>>;
pub type LocalizerRef = Rc<dyn Localizer<'static>>;
pub type LanguageRequesterRef = Rc<RefCell<dyn LanguageRequester<'static>>>;

pub enum Msg {
    StateChanged(Rc<CosterState>, CosterEvent),
}

pub struct Model {
    language_requester: LanguageRequesterRef,
    localizer: LocalizerRef,
    link: ComponentLink<Self>,
    state_store: StateStoreRef,
    _state_callback: yew_state::Callback<CosterState, CosterEvent>,
}

impl Model {
    fn page(&self, inner: Html) -> Html {
        html! {
            <Page
                state_store = self.state_store.clone()
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
        // let log_middleware = SimpleLoggerMiddleware::new().log_level(LogLevel::Debug);
        let log_middleware = WebLoggerMiddleware::new().log_level(LogLevel::Log);
        state_store.add_middleware(log_middleware);

        let route_middleware = RouteMiddleware::new(state_store.clone());
        state_store.add_middleware(route_middleware);

        let mut language_requester: WebLanguageRequester<'static> = WebLanguageRequester::new();
        let localizer = DefaultLocalizer::new(&*LANGUAGE_LOADER, &TRANSLATIONS);
        let localizer_ref: Rc<dyn Localizer<'static>> = Rc::new(localizer);
        language_requester.add_listener(Rc::downgrade(&localizer_ref));

        // Manually check the currently requested system language,
        // and update the listeners. When the system language changes,
        // this will automatically be triggered.
        language_requester.poll().unwrap();

        let language_requester_ref = Rc::new(RefCell::new(language_requester));
        let localize_middleware = LocalizeMiddleware::new(language_requester_ref.clone());
        state_store.add_middleware(localize_middleware);

        let state_store_clone = state_store.clone();

        // TODO: this has a problem where if the user changes
        // something before the database loads (or any other event
        // attempts to change something), it will be overridden, and
        // the change will be lost. #18
        wasm_bindgen_futures::spawn_local(async move {
            let database_result: Result<kvdb_web::Database, _> =
                kvdb_web::Database::open("CosterState".to_string(), 1).await;
            match database_result {
                Ok(database) => {
                    let database_middleware = DatabaseMiddleware::new(database);

                    state_store_clone.add_middleware(database_middleware);
                    state_store_clone.dispatch(CosterAction::LoadDatabase)
                }
                Err(error) => error!("Error opening database: {}", error),
            }
        });

        let state_callback = link
            .callback(|(state, event)| Msg::StateChanged(state, event))
            .into();

        state_store.subscribe_events(
            &state_callback,
            vec![CosterEvent::LanguageChanged, CosterEvent::RouteChanged],
        );

        state_store.dispatch(RouteAction::PollBrowserRoute);

        Model {
            language_requester: language_requester_ref,
            localizer: localizer_ref,
            link,
            state_store,
            _state_callback: state_callback,
        }
    }

    fn update(&mut self, msg: Msg) -> ShouldRender {
        match msg {
            Msg::StateChanged(_state, event) => match event {
                CosterEvent::LanguageChanged => {
                    // if let Some(storage) = &mut self.storage {
                    //     debug!(
                    //         "Model::update storing user-selected-language: {:?}",
                    //         state.selected_language
                    //     );

                    //     storage.store("user-selected-language", Ok(state.selected_language.to_string()));
                    // }
                    // debug!("Language changed in coster::lib {:?}", state.selected_language);
                    true
                }
                CosterEvent::RouteChanged => true,
                _ => false,
            },
        }
    }

    fn view(&self) -> Html {
        let state = self.state_store.state();
        let route_match_node = match &state.route {
            RouteType::Valid(AppRoute::CostingTab) => self.page(centered(
                html! {<CostingTab state_store=self.state_store.clone()/>},
            )),
            RouteType::Valid(AppRoute::NewCostingTab) => self.page(centered(
                html! {<NewCostingTab state_store=self.state_store.clone()/>},
            )),
            RouteType::Valid(AppRoute::Help) => {
                self.page(html! { <h1 class="title is-1">{ tr!("Help for Coster") }</h1> })
            }
            RouteType::Valid(AppRoute::About) => {
                self.page(html! { <h1 class="title is-1">{ tr!("About Coster") }</h1> })
            }
            RouteType::Valid(AppRoute::Index) => {
                if state.route.path() == "/" {
                    self.page(centered(
                        html! {<CostingTabList state_store=self.state_store.clone()/>},
                    ))
                } else {
                    debug!(target: "gui::router", "Detected Invalid Route: {:?}", state.route);
                    VNode::from("404")
                }
            }
            RouteType::Invalid(route) => {
                debug!(target: "gui::router", "Detected Invalid Route: {:?}", route);
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
