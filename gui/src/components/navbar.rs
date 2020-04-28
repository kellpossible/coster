use super::select::Select;
use crate::{bulma, AppRoute, AppRouterRef, LocalizerRef};

use tr::tr;
use unic_langid::LanguageIdentifier;
use yew::{html, Callback, Component, ComponentLink, Html, Properties, ShouldRender};

pub struct Navbar {
    burger_menu_active: bool,
    props: Props,
    link: ComponentLink<Self>,
}

pub enum Msg {
    ToggleBurgerMenu,
    ToIndex,
    ToHelp,
    ToAbout,
}

#[derive(Clone, Properties)]
pub struct Props {
    pub router: AppRouterRef,
    #[prop_or_default]
    pub on_language_change: Callback<LanguageIdentifier>,
    pub localizer: LocalizerRef,
    pub lang: unic_langid::LanguageIdentifier,
}

impl PartialEq for Props {
    fn eq(&self, other: &Props) -> bool {
        self.on_language_change == other.on_language_change && self.lang == other.lang
    }
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
            },
            Msg::ToIndex => {
                self.burger_menu_active = false;
                self.props.router.borrow_mut().set_route(AppRoute::Index);
            },
            Msg::ToAbout => {
                self.burger_menu_active = false;
                self.props.router.borrow_mut().set_route(AppRoute::About);
            },
            Msg::ToHelp => {
                self.burger_menu_active = false;
                self.props.router.borrow_mut().set_route(AppRoute::Help);
            }
        }
        true
    }

    fn view(&self) -> Html {
        let languages = self.props.localizer.available_languages().unwrap();
        let default_language = self.props.localizer.language_loader().current_language();

        let select_icon_props = bulma::components::icon::Props {
            color: Some(bulma::Color::Info),
            span_class: vec![],
            class: vec!["fas".to_string(), "fa-globe".to_string()],
        };

        let onclick_burger = self.link.callback(|_| Msg::ToggleBurgerMenu);
        let onclick_coster_index = self.link.callback(|_| Msg::ToIndex);
        let onclick_help = self.link.callback(|_| Msg::ToHelp);
        let onclick_about = self.link.callback(|_| Msg::ToAbout);

        let mut burger_classes = vec!["navbar-burger"];
        let mut menu_classes = vec!["navbar-menu"];

        if self.burger_menu_active {
            burger_classes.push("is-active");
            menu_classes.push("is-active");
        }

        html! {
            <nav class="navbar is-dark" role="navigation" aria-label="main navigation">
                <div class="navbar-brand">
                    <a class="navbar-item" onclick=onclick_coster_index>
                        { tr!("Coster") }
                    </a>
                    <a role="button" class=burger_classes aria-label="menu" aria-expanded="false" onclick=onclick_burger>
                        <span aria-hidden="true"></span>
                        <span aria-hidden="true"></span>
                        <span aria-hidden="true"></span>
                    </a>
                </div>

                <div id="navbarBasicExample" class=menu_classes>
                    <div class="navbar-start">
                        <a class="navbar-item" onclick=onclick_help>
                            { tr!("Help") }
                        </a>

                        <a class="navbar-item" onclick=onclick_about>
                            { tr!("About") }
                        </a>
                    </div>

                    <div class="navbar-end">
                        <div class="navbar-item">
                            <Select<LanguageIdentifier>
                                size=bulma::Size::Big
                                selected=default_language
                                options=languages
                                onchange=self.props.on_language_change.clone()
                                icon_props=select_icon_props
                                />
                        </div>
                    </div>
                </div>
            </nav>
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        if self.props != props {
            self.props = props;
            true
        } else {
            false
        }
    }
}
