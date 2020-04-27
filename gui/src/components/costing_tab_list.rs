use std::cell::RefCell;
use std::rc::Rc;

use commodity::CommodityType;
use costing::{Tab, TabID};
use tr::tr;
use yew::{html, Component, ComponentLink, Html, Properties, ShouldRender};

pub struct CostingTabList {
    tab: RefCell<Tab>,
    props: Props,
    link: ComponentLink<Self>,
}

#[derive(Clone, Properties, PartialEq)]
pub struct Props {
    pub lang: unic_langid::LanguageIdentifier,
}

impl Component for CostingTabList {
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
        CostingTabList { tab, props, link }
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
                        <h3 class="title is-3">{ tr!("Your Tabs") }</h3>
                    </div>
                </div>
                <div class="level-right">
                    <div class="level-item">
                        <button class="button is-success">{ tr!("New Tab") }</button>
                    </div>
                </div>
            </nav>
        }
    }
}
