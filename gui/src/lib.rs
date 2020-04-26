#![recursion_limit = "512"]

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};

use costing::TabID;

use i18n_embed::{
    language_loader, DefaultLocalizer, I18nEmbed, LanguageRequester, Localizer,
    WebLanguageRequester,
};
use rust_embed::RustEmbed;
use yew::virtual_dom::VNode;
use yew::{html, Component, ComponentLink, Html, ShouldRender};
use yew_router::{route::Route, service::RouteService, Switch};

use lazy_static::lazy_static;
use log;
use log::debug;
use tr::tr;
use wasm_bindgen::prelude::*;

pub mod bulma;
mod components;

use components::costing_tab_list::CostingTabList;
use components::navbar::Navbar;
use components::pages::{CostingTabListPage, CostingTabPage};

#[derive(RustEmbed, I18nEmbed)]
#[folder = "i18n/mo"]
struct Translations;

language_loader!(WebLanguageLoader);

lazy_static! {
    static ref LANGUAGE_LOADER: WebLanguageLoader = WebLanguageLoader::new();
}

static TRANSLATIONS: Translations = Translations {};

#[derive(Switch, Debug, Clone)]
pub enum AppRoute {
    #[to = "/tab"]
    CostingTab,
    #[to = "/"]
    Index,
}

pub enum Msg {
    RouteChanged(Route<()>),
    ChangeRoute(AppRoute),
}

pub struct Model {
    language_requester: Rc<RefCell<dyn LanguageRequester<'static>>>,
    localizer: Rc<Box<dyn Localizer<'static>>>,
    route_service: RouteService<()>,
    route: Route<()>,
    link: ComponentLink<Self>,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let mut language_requester: WebLanguageRequester<'static> = WebLanguageRequester::new();

        // language_requester.set_languge_override(Some("en-GB".parse().unwrap())).unwrap();

        let localizer = DefaultLocalizer::new(&*LANGUAGE_LOADER, &TRANSLATIONS);

        let localizer_rc: Rc<Box<dyn Localizer<'static>>> = Rc::new(Box::from(localizer));
        language_requester.add_listener(&localizer_rc);

        // Manually check the currently requested system language,
        // and update the listeners. When the system language changes,
        // this will automatically be triggered.
        language_requester.poll().unwrap();

        let language_requester_rc: Rc<RefCell<dyn LanguageRequester<'static>>> =
            Rc::new(RefCell::from(language_requester));

        let route_service: RouteService<()> = RouteService::new();
        let route = route_service.get_route();

        debug!(target: "gui::router", "Initial Route: {:?}", route);

        Model {
            link,
            language_requester: language_requester_rc,
            route_service,
            route,
            localizer: localizer_rc,
        }
    }

    fn update(&mut self, msg: Msg) -> ShouldRender {
        match msg {
            Msg::RouteChanged(route) => self.route = route,
            Msg::ChangeRoute(route) => {
                // This might be derived in the future
                let route_string = match route {
                    AppRoute::Index => "/".to_string(),
                    AppRoute::CostingTab => "/tab".to_string(),
                };
                self.route_service.set_route(&route_string, ());
                self.route = Route {
                    route: route_string,
                    state: (),
                };
            }
        }
        true
    }

    fn view(&self) -> Html {
        // costing_tab_page_html = html!{<CostingTabPage localizer=self.localizer.clone() language_requester=self.language_requester.clone()/>};

        let route_match_node = match AppRoute::switch(self.route.clone()) {
            Some(AppRoute::CostingTab) => {
                debug!(target: "gui::router", "Detected CostingTab Route: {:?}", self.route);
                //TODO: implement Page content properly so CostingTabPage can work
                html! { <h1>{"CostingTabPage"}</h1> }
                // html! {<CostingTabPage localizer=self.localizer.clone() language_requester=self.language_requester.clone()/>}
            }
            Some(AppRoute::Index) => {
                debug!(target: "gui::router", "Detected CostingTabListPage Route: {:?}", self.route);
                html! {<CostingTabListPage localizer=self.localizer.clone() language_requester=self.language_requester.clone()/>}
            }
            _ => {
                //TODO: currently this route isn't working
                debug!(target: "gui::router", "Detected Invalid Route: {:?}", self.route);
                VNode::from("404")
            },
        };
        

        html! {
            <>
            {
                route_match_node
            }
            </>
            // <Router<AppRoute, ()> render = Router::render(|switch: AppRoute| {
            //
            // })/>
            // <Router>
            //     <Route matcher=route!("/a/{}" CaseInsensitive) render=component::<AModel>() />
            //     <Route matcher=route!("/c") render=component::<CModel>() />
            // </Router>
            // <Router<AppRoute, ()>
            //     render = Router::render(|switch: AppRoute| {
            //         match switch {
            //             AppRoute::CostingTab=> html!{<CostingTabPage localizer=self.localizer.clone() language_requester=self.language_requester.clone()/>},
            //             AppRoute::Index => html!{<></>},
            //         }
            //     })
            // />
        }
    }

    fn change(&mut self, _: Self::Properties) -> ShouldRender {
        false
    }
}

// Called when the wasm module is instantiated
#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    #[cfg(feature = "console_log")]
    console_log::init_with_level(log::Level::Debug).unwrap();

    yew::start_app::<Model>();
    Ok(())
}
