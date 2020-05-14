use std::cell::RefCell;
use std::rc::Rc;

use commodity::CommodityType;
use costing::Tab;
use tr::tr;
use yew::{html, Component, ComponentLink, Html, Properties, ShouldRender};
use crate::state::StateStore;

pub struct CostingTab {
    tab: RefCell<Tab>,
    props: Props,
    link: ComponentLink<Self>,
}

#[derive(Clone, Properties)]
pub struct Props {
    pub state_store: StateStore,
}

impl PartialEq for Props {
    fn eq(&self, other: &Self) -> bool {
        StateStore::ptr_eq(&self.state_store, &other.state_store)
    }
}

impl Component for CostingTab {
    type Message = ();
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let tab = RefCell::new(Tab::new(
            0,
            "Test Tab",
            Rc::new(CommodityType::from_currency_alpha3("AUD").unwrap()),
            vec![],
            vec![],
        ));
        CostingTab { tab, props, link }
    }

    fn update(&mut self, _: Self::Message) -> ShouldRender {
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

        html! {
            <nav class="level">
                <div class="level-left">
                    <div class="level-item">
                        <h3 class="title is-3">{ tab.name.clone() }</h3>
                    </div>
                </div>
                <div class="level-right">
                    <div class="level-item">
                        <button class="button is-success">{ tr!("Add Expense") }</button>
                    </div>
                </div>
            </nav>
        }
    }
}
