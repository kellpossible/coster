use std::rc::Rc;

use i18n_embed::{
    I18nEmbed, LanguageLoader, language_loader, WebLanguageRequester,
    LanguageRequester, DefaultLocalizer, Localizer};
use rust_embed::RustEmbed;
use yew::{html, Component, ComponentLink, Html, ShouldRender, components::{Select, select}};

use wasm_bindgen::prelude::*;
use unic_langid::LanguageIdentifier;
use log;
use log::debug;
use lazy_static::lazy_static;

mod test;
mod components;

use components::ClickerButton;

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
}

pub struct Model {
    language_requester: WebLanguageRequester<'static>,
    localizer: Rc<Box<dyn Localizer<'static>>>,
    link: ComponentLink<Self>,
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
        }
    }

    fn update(&mut self, msg: LanguageMsg) -> ShouldRender {
        match msg {
            LanguageMsg::Select(language) => {
                self.language_requester.set_languge_override(Some(language)).unwrap();
                self.language_requester.poll().unwrap();
                self.change(());
            }
        }

        true
    }

    fn view(&self) -> Html {
        let languages = self.localizer.available_languages().unwrap();
        let default_language = self.localizer.language_loader().current_language();

        html! {
            <html>
                <body>
                    <ClickerButton />
                    <Select<LanguageIdentifier> selected=default_language, options=languages onchange=self.link.callback(|selection| {
                        debug!("GUI Language Selection: {}", selection);
                        LanguageMsg::Select(selection)
                    }) />
                </body>
            </html>
        }
    }
    
}


// Called when the wasm module is instantiated
#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    console_log::init_with_level(log::Level::Debug).unwrap();

    yew::start_app::<Model>();
    Ok(())
}
