use crate::{components::navbar::Navbar, state::StateStoreRef, LanguageRequesterRef};

use std::rc::Rc;
use yew::{
    html, html::Renderable, Children, Component, ComponentLink, Html, Properties, ShouldRender,
};

pub struct Page {
    props: Props,
    _link: ComponentLink<Self>,
}

#[derive(Clone, Properties)]
pub struct Props {
    pub state_store: StateStoreRef,
    pub language_requester: LanguageRequesterRef,
    #[prop_or_default]
    pub children: Children,
}

impl PartialEq for Props {
    fn eq(&self, other: &Self) -> bool {
        self.state_store == other.state_store
            && Rc::ptr_eq(&self.language_requester, &other.language_requester)
            && self.children == other.children
    }
}

impl Component for Page {
    type Message = ();
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Page { props, _link: link }
    }

    fn update(&mut self, _: ()) -> ShouldRender {
        false
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
        html! {
            <>
                <Navbar
                    state_store = self.props.state_store.clone()
                    language_requester = self.props.language_requester.clone()/>
                { self.props.children.clone() }
            </>
        }
    }
}
