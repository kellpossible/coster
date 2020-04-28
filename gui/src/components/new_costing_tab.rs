use crate::components::select::Select;

use std::cell::RefCell;
use std::rc::Rc;

use commodity::CommodityType;
use costing::{Tab};
use tr::tr;
use yew::{html, Component, ComponentLink, Html, Properties, ShouldRender};

pub struct NewCostingTab {
    props: Props,
    link: ComponentLink<Self>,
    currencies: Vec<CommodityType>
}

#[derive(Clone, Properties, PartialEq)]
pub struct Props {
    pub lang: unic_langid::LanguageIdentifier,
}

impl Component for NewCostingTab {
    type Message = ();
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let mut currencies = commodity::all_iso4217_currencies();
        currencies.sort_by(|a, b| a.id.cmp(&b.id));

        NewCostingTab { props, link, currencies}
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
        let default_currency = CommodityType::from_currency_alpha3("AUD").unwrap();
        html! {
            <>
                <nav class="level">
                    <div class="level-left">
                        <div class="level-item">
                            <h3 class="title is-3">{ tr!("New Tab") }</h3>
                        </div>
                    </div>
                </nav>

                <div class="card">
                    <form>
                        <div class="field">
                            <label class="label">{ tr!("Name") }</label>
                            <div class="control">
                                <input class="input" type="text" placeholder=tr!("Tab Name")/>
                            </div>
                        </div>
                        <div class="field">
                            <label class="label">{ tr!("Working Currency") }</label>
                            <div class="control">
                                <Select<CommodityType>
                                        selected=default_currency
                                        options=self.currencies.clone()
                                        />
                            </div>
                        </div>

                        <div class="field is-grouped">
                            <div class="control">
                                <button class="button is-link">{ tr!("Create") }</button>
                            </div>
                            <div class="control">
                                <button class="button is-link is-light">{ tr!("Cancel") }</button>
                            </div>
                        </div>
                    </form>
                </div>
            </>
        }
    }
}
