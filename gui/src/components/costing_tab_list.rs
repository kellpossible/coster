use crate::{
    state::{middleware::route::RouteStoreRef, StateStoreRef},
    AppRoute,
};

use std::cell::RefCell;
use std::rc::Rc;

use commodity::CommodityType;
use costing::Tab;
use log::debug;
use tr::tr;
use yew::MouseEvent;
use yew::{html, Component, ComponentLink, Html, Properties, ShouldRender};

pub struct CostingTabList {
    tab: RefCell<Tab>,
    props: Props,
    link: ComponentLink<Self>,
}

pub enum Msg {
    NewCostingTab,
}

#[derive(Clone, Properties, PartialEq)]
pub struct Props {
    pub state_store: StateStoreRef,
}

impl Component for CostingTabList {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let tab = RefCell::new(Tab::new(
            0,
            "Test Tab",
            Rc::new(CommodityType::from_currency_alpha3("AUD").unwrap()),
            vec![],
            vec![],
        ));
        CostingTabList { tab, props, link }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::NewCostingTab => {
                self.props.state_store.change_route(AppRoute::NewCostingTab);
            }
        }
        true
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
        let tab = self.tab.borrow();

        let new_tab_handler = self.link.callback(|msg: MouseEvent| {
            debug!("Clicked New Tab Button: {:?}", msg);
            Msg::NewCostingTab
        });

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
