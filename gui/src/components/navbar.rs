use crate::{
    state::{
        middleware::localize::LocalizeStore,
        StateCallback, StateStoreRef,
    },
    AppRoute, LanguageRequesterRef,
};
use yew_bulma::components::select::Select;

use std::rc::Rc;
use tr::tr;
use unic_langid::LanguageIdentifier;
use yew::{html, Component, ComponentLink, Html, Properties, ShouldRender};
use switch_router_middleware::RouteStore;

pub struct Navbar {
    burger_menu_active: bool,
    props: Props,
    link: ComponentLink<Self>,
    available_languages: Vec<LanguageIdentifier>,
    _language_changed_callback: StateCallback,
}

#[derive(Clone)]
pub enum Msg {
    ToggleBurgerMenu,
    ToIndex,
    ToHelp,
    ToAbout,
    SelectLanguage(LanguageIdentifier),
    LanguageChanged,
}

#[derive(Clone, Properties)]
pub struct Props {
    pub state_store: StateStoreRef,
    pub language_requester: LanguageRequesterRef,
}

impl PartialEq for Props {
    fn eq(&self, other: &Props) -> bool {
        self.state_store == other.state_store
            && Rc::ptr_eq(&self.language_requester, &other.language_requester)
    }
}

impl Component for Navbar {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Props, link: ComponentLink<Self>) -> Self {
        let mut available_languages = props
            .language_requester
            .borrow()
            .available_languages()
            .unwrap();

        available_languages.sort();

        let callback = props
            .state_store
            .subscribe_language_changed(&link, Msg::LanguageChanged);

        Navbar {
            burger_menu_active: false,
            props,
            link,
            available_languages,
            _language_changed_callback: callback,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::ToggleBurgerMenu => {
                self.burger_menu_active = !self.burger_menu_active;
                true
            }
            Msg::ToIndex => {
                self.burger_menu_active = false;
                self.props.state_store.change_route(AppRoute::Index);
                true
            }
            Msg::ToAbout => {
                self.burger_menu_active = false;
                self.props.state_store.change_route(AppRoute::About);
                true
            }
            Msg::ToHelp => {
                self.burger_menu_active = false;
                self.props.state_store.change_route(AppRoute::Help);
                true
            }
            Msg::SelectLanguage(language) => {
                self.props
                    .state_store
                    .change_selected_language(Some(language), true);
                true
            }
            Msg::LanguageChanged => true,
        }
    }

    fn view(&self) -> Html {
        let current_languages = self.props.language_requester.borrow().current_languages();
        let current_language = current_languages
            .get("gui")
            .expect("expected there to be a current language for the \"gui\" module/domain");

        let on_language_change = self.link.callback(Msg::SelectLanguage);

        let select_icon_props = yew_bulma::components::icon::Props {
            color: Some(yew_bulma::classes::Color::Info),
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
                                size=yew_bulma::classes::Size::Big
                                selected=current_language
                                options=self.available_languages.clone()
                                onchange=on_language_change
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
