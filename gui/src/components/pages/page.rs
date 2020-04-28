use crate::{components::navbar::Navbar, AppRouterRef, LanguageRequesterRef, LocalizerRef};

use log::debug;
use unic_langid::LanguageIdentifier;
use yew::{
    html, html::Renderable, Callback, Children, Component, ComponentLink, Html, Properties,
    ShouldRender,
};

pub struct Page {
    props: Props,
    link: ComponentLink<Self>,
}

pub enum LanguageMsg {
    Select(unic_langid::LanguageIdentifier),
}

#[derive(Clone, Properties)]
pub struct Props {
    pub router: AppRouterRef,
    pub language_requester: LanguageRequesterRef,
    pub localizer: LocalizerRef,
    #[prop_or_default]
    pub on_language_change: Callback<LanguageIdentifier>,
    pub children: Children,
}

impl PartialEq for Props {
    fn eq(&self, other: &Props) -> bool {
        self.localizer.language_loader().current_language()
            == other.localizer.language_loader().current_language()
            && self.language_requester.borrow().requested_languages()
                == other.language_requester.borrow().requested_languages()
            && self.children == other.children
            && self.on_language_change == other.on_language_change
    }
}

impl Component for Page {
    type Message = LanguageMsg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Page { props, link }
    }

    fn update(&mut self, msg: LanguageMsg) -> ShouldRender {
        match msg {
            LanguageMsg::Select(language) => {
                self.props
                    .language_requester
                    .borrow_mut()
                    .set_languge_override(Some(language.clone()))
                    .unwrap();
                self.props.language_requester.borrow_mut().poll().unwrap();
                self.change(self.props.clone());
                self.props.on_language_change.emit(language);
                true
            }
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

    fn view(&self) -> Html {
        let lang = self.props.localizer.language_loader().current_language();

        html! {
            <>
                <Navbar
                    router = self.props.router.clone()
                    lang = lang.clone()
                    localizer = self.props.localizer.clone()
                    on_language_change = self.link.callback(|selection| {
                        debug!("GUI Language Selection: {}", selection);
                        LanguageMsg::Select(selection)
                    })/>
                { self.props.children.render() }
            </>
        }
    }
}