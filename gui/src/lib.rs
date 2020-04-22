#![recursion_limit="512"]

use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};

use i18n_embed::{
    I18nEmbed, language_loader, WebLanguageRequester,
    LanguageRequester, DefaultLocalizer, Localizer};
use rust_embed::RustEmbed;
use yew::{html, Component, ComponentLink, Html, ShouldRender, Properties};

use wasm_bindgen::prelude::*;
use log;
use log::debug;
use lazy_static::lazy_static;
use tr::tr;

mod components;
pub mod bulma;

use components::ClickerButton;
use components::navbar::Navbar;

#[derive(RustEmbed, I18nEmbed)]
#[folder = "i18n/mo"]
struct Translations;

language_loader!(WebLanguageLoader);

lazy_static! {
    static ref LANGUAGE_LOADER: WebLanguageLoader = WebLanguageLoader::new();
}

static TRANSLATIONS: Translations = Translations {};

pub enum LanguageMsg {
    Select(unic_langid::LanguageIdentifier),
    Rerender,
}

pub struct Model {
    language_requester: WebLanguageRequester<'static>,
    localizer: Rc<Box<dyn Localizer<'static>>>,
    rerender: AtomicBool,
    link: ComponentLink<Self>,
}

impl Model {
    fn localized_html(&self, localized: Html) -> Html {
        if self.rerender.load(Ordering::Relaxed) {
            self.link.send_message(LanguageMsg::Rerender);
            html! {
                <div>
                </div>
            }
        } else {
            html! {
                <div>
                { localized }
                </div>
            }
        }
    }
}

impl Component for Model {
    type Message = LanguageMsg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let mut language_requester = WebLanguageRequester::new();
        // language_requester.set_languge_override(Some("en-GB".parse().unwrap())).unwrap();

        let localizer = DefaultLocalizer::new(
            &*LANGUAGE_LOADER,
            &TRANSLATIONS,
        );

        let localizer_rc: Rc<Box<dyn Localizer<'static>>> = Rc::new(Box::from(localizer));
        language_requester.add_listener(&localizer_rc);

        // Manually check the currently requested system language,
        // and update the listeners. When the system language changes,
        // this will automatically be triggered.
        language_requester.poll().unwrap();

        Model { 
            link,
            language_requester,
            localizer: localizer_rc,
            rerender: AtomicBool::new(false),
        }
    }

    fn update(&mut self, msg: LanguageMsg) -> ShouldRender {
        match msg {
            LanguageMsg::Select(language) => {
                self.language_requester.set_languge_override(Some(language)).unwrap();
                self.language_requester.poll().unwrap();
                self.change(());
                self.rerender.store(true, Ordering::Relaxed);
                true
            },
            LanguageMsg::Rerender => {
                self.rerender.store(false, Ordering::Relaxed);
                true
            }
        }
    }

    fn view(&self) -> Html {
        let navbar_brand = html! {
            <a class="navbar-item" href="/">
                { tr!("Coster") }
            </a>
        };

        let lang = self.localizer.language_loader().current_language();

        debug!("Rendering Root");

        html! {
            <>
                <Navbar lang=lang.clone() brand=navbar_brand localizer=self.localizer.clone() on_language_change = self.link.callback(|selection| {
                    debug!("GUI Language Selection: {}", selection);
                    LanguageMsg::Select(selection)
                })/>
                <ClickerButton lang=lang />
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
    console_log::init_with_level(log::Level::Debug).unwrap();

    yew::start_app::<Model>();
    Ok(())
}
