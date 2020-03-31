use std::path::Path;
use std::path;

use fluent_langneg::convert_vec_str_to_langids_lossy;
use fluent_langneg::negotiate_languages;
use fluent_langneg::NegotiationStrategy;
use gettext::Catalog;
use rust_embed::RustEmbed;
use tr::{set_translator, tr};
use unic_langid::LanguageIdentifier;
use wasm_bindgen::prelude::*;
use web_sys::console;
use yew::{html, Component, ComponentLink, Html, ShouldRender};
use itertools::Itertools;

mod test;

#[derive(RustEmbed)]
#[folder = "i18n/mo"]
struct Translations;

pub struct Model {
    link: ComponentLink<Self>,
}

pub enum Msg {
    Click,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        Model { link }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Click => {
                console::log_1(&tr!("Hello World, this is me!").into());
            }
        }
        true
    }

    fn view(&self) -> Html {
        html! {
            <div>
                <button class="button" onclick=self.link.callback(|_| Msg::Click)>{ tr!("Click") }</button>
            </div>
        }
    }
}

fn available_languages() -> Vec<String> {
    let mut languages: Vec<String> = Translations::iter()
        .map(|filename_cow| filename_cow.to_string())
        .filter_map(|filename| {
            let path: &Path = Path::new(&filename);

            console::log_1(&format!("Path: {:?}", path).into());

            let components: Vec<path::Component> = path
                .components()
                .collect();

            console::log_1(&format!("components: {:?}", components).into());

            let component: Option<String> = match components.get(0) {
                Some(component) => {
                    match component {
                        path::Component::Normal(s) => {
                            Some(s.to_str().expect("path should be valid utf-8").to_string())
                        },
                        _ => None,
                    }
                }
                _ => None,
            };

            component
        })
        .unique()
        .collect();

    languages.insert(0, String::from(DEFAULT_LANGUAGE_ID));
    return languages;
}

const DEFAULT_LANGUAGE_ID: &str = "en-GB";

pub fn setup_translations() {
    console::log_1(&"Setting the translator version 3!".into());
    let window = web_sys::window().expect("no global `window` exists");
    let navigator = window.navigator();
    let languages = navigator.languages();

    let requested_languages = convert_vec_str_to_langids_lossy(languages.iter().map(|js_value| {
        js_value
            .as_string()
            .expect("language value should be a string.")
    }));
    
    let available_languages: Vec<LanguageIdentifier> = convert_vec_str_to_langids_lossy(available_languages());
    let default_language: LanguageIdentifier = DEFAULT_LANGUAGE_ID.parse().expect("Parsing langid failed.");

    let supported_languages = negotiate_languages(
        &requested_languages,
        &available_languages,
        Some(&default_language),
        NegotiationStrategy::Filtering,
    );

    console::log_1(&format!("Requested Languages: {:?}", requested_languages).into());
    console::log_1(&format!("Available Languages: {:?}", available_languages).into());
    console::log_1(&format!("Supported Languages: {:?}", supported_languages).into());

    match supported_languages.get(0) {
        Some(language_id) => {
            if language_id != &&default_language {
                let language_id_string = language_id.to_string();
                let f = Translations::get(format!("{}/gui.mo", language_id_string).as_ref())
                    .expect("could not read the file");
                let catalog = Catalog::parse(&*f).expect("could not parse the catalog");
                set_translator!(catalog);
            }
        }
        None => {
            // do nothing
        }
    }

    console::log_1(&"Completed setting translations!".into());
}

// Called when the wasm module is instantiated
#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    setup_translations();
    yew::start_app::<Model>();
    Ok(())
}
