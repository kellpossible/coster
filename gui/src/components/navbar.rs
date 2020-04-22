use super::select::Select;
use crate::bulma;

use std::rc::Rc;

use yew::{html, Component, ComponentLink, Html, ShouldRender, Properties, Callback};
use tr::tr;
use unic_langid::LanguageIdentifier;
use i18n_embed::Localizer;
use log::debug;


pub struct Navbar {
    burger_menu_active: bool,
    props: Props,
    link: ComponentLink<Self>,
}

pub enum Msg {
    ToggleBurgerMenu,
}

#[derive(Clone, Properties)]
pub struct Props {
    #[prop_or_default]
    pub brand: Option<Html>,
    #[prop_or_default]
    pub on_language_change: Callback<LanguageIdentifier>,
    pub localizer: Rc<Box<dyn Localizer<'static>>>,
}

impl Component for Navbar {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Navbar {
            burger_menu_active: false,
            props,
            link,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::ToggleBurgerMenu => {
                self.burger_menu_active = !self.burger_menu_active;
                true
            }
                
        }
    }

    fn view(&self) -> Html {
        let languages = self.props.localizer.available_languages().unwrap();
        let default_language = self.props.localizer.language_loader().current_language();
        
        let select_icon_props = bulma::components::icon::Props {
            color: Some(bulma::Color::Info),
            span_class: vec![],
            class: vec!["fas".to_string(), "fa-globe".to_string()]
        };

        debug!("Rendering Navbar");

        let onclick_burger = self.link.callback(|_| Msg::ToggleBurgerMenu );

        let mut burger_classes = vec!["navbar-burger"];
        let mut menu_classes = vec!["navbar-menu"];

        if self.burger_menu_active {
            burger_classes.push("is-active");
            menu_classes.push("is-active");
        }

        html! {
            <nav class="navbar is-dark" role="navigation" aria-label="main navigation">
                {
                    if self.props.brand.is_some() {
                        html!{
                            <div class="navbar-brand">
                                { self.props.brand.as_ref().unwrap().clone() }
                                <a role="button" class=burger_classes aria-label="menu" aria-expanded="false" onclick=onclick_burger>
                                    <span aria-hidden="true"></span>
                                    <span aria-hidden="true"></span>
                                    <span aria-hidden="true"></span>
                                </a>
                            </div>
                        }
                    } else {
                        html! {}
                    }
                }

                <div id="navbarBasicExample" class=menu_classes>
                    <div class="navbar-start">
                        <a class="navbar-item">
                            { tr!("Home") }
                        </a>

                        <a class="navbar-item">
                            { tr!("Documentation") }
                        </a>

                        <div class="navbar-item has-dropdown is-hoverable">
                            <a class="navbar-link">
                            { tr!("More") }
                            </a>

                            <div class="navbar-dropdown">
                                <a class="navbar-item">
                                { tr!("About") }
                                </a>
                                <a class="navbar-item">
                                { tr!("Jobs") }
                                </a>
                                <a class="navbar-item">
                                { tr!("Contact") }
                                </a>
                                <hr class="navbar-divider"/>
                                <a class="navbar-item">
                                { tr!("Report an issue") }
                                </a>
                            </div>
                        </div>
                    </div>

                    <div class="navbar-end">
                        <div class="navbar-item">
                            // <div class="buttons">
                                <Select<LanguageIdentifier> 
                                size=bulma::Size::Big 
                                selected=default_language 
                                options=languages 
                                onchange=self.props.on_language_change.clone()
                                icon_props=select_icon_props
                                />
                            // </div>
                        </div>
                    </div>
                </div>
            </nav>
        }
    }
}