use crate::state::middleware::localize::LocalizeStore;
use crate::{
    state::{middleware::route::RouteStore, StateCallback, StateStoreRef, CosterEvent},
    AppRoute,
};

use std::cell::RefCell;
use std::rc::Rc;

use commodity::CommodityType;
use costing::Tab;
use tr::tr;
use yew::MouseEvent;
use yew::{html, Component, ComponentLink, Html, Properties, ShouldRender};
use uuid::Uuid;

pub struct CostingTabList {
    props: Props,
    link: ComponentLink<Self>,
    _language_changed_callback: StateCallback,
    _tabs_changed_callback: StateCallback,
}

#[derive(Clone)]
pub enum Msg {
    NewCostingTab,
    LanguageChanged,
    TabsChanged,
}

#[derive(Clone, Properties, PartialEq)]
pub struct Props {
    pub state_store: StateStoreRef,
}

impl Component for CostingTabList {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let language_changed_callback = props
            .state_store
            .subscribe_language_changed(&link, Msg::LanguageChanged);

        let tabs_changed_callback = link.callback(|(_store, _event)| Msg::TabsChanged).into();

        props.state_store.subscribe_event(&tabs_changed_callback, CosterEvent::TabsChanged);

        CostingTabList {
            props,
            link,
            _language_changed_callback: language_changed_callback,
            _tabs_changed_callback: tabs_changed_callback,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::NewCostingTab => {
                self.props.state_store.change_route(AppRoute::NewCostingTab);
                true
            }
            Msg::LanguageChanged => true,
            Msg::TabsChanged => true,
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
        let new_tab_handler = self.link.callback(|msg: MouseEvent| Msg::NewCostingTab);

        html! {
            <nav class="level">
                <div class="level-left">
                    <div class="level-item">
                        <h3 class="title is-3">{ tr!("Your Tabs") }</h3>
                    </div>
                </div>
                <div class="level-right">
                    <div class="level-item">
                        <button class="button is-success" onclick = new_tab_handler>{ tr!("New Tab") }</button>
                    </div>
                </div>
            </nav>
        }
    }
}
