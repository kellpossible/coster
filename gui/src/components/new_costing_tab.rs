use crate::{AppRouterRef, bulma::components::Select, AppRoute};

use commodity::CommodityType;
use tr::tr;
use yew::{html, Component, ComponentLink, Html, Properties, ShouldRender, ChangeData};
use log::info;
use validator::{Validate, ValidationErrors, ValidationError};


#[derive(Debug, Validate)]
pub struct FormData {
    #[validate(length(min = 1, code="not-empty"))]
    name: String,
    working_currency: CommodityType,
    validation_errors: ValidationErrors,
}

fn validation_message(error: &ValidationError) -> String {
    match error.code.as_ref() {
        "not-empty" => tr!("Field should not be empty"),
        _ => error.to_string(),
    }
}

fn validation_messages(errors: &Vec<ValidationError>) -> String {
    let error_strings: Vec<String> = errors.iter().map(|error| validation_message(error)).collect();
    error_strings.join(" and ")
}

impl FormData {
    pub fn new() -> Self {
        Self {
            name: "".to_string(),
            working_currency: CommodityType::from_currency_alpha3("AUD").unwrap(),
            validation_errors: ValidationErrors::new(),
        }
    }

    pub fn run_validation(&mut self) {
        if let Err(errors) = self.validate() {
            self.validation_errors = errors;
        } else if !self.validation_errors.is_empty() {
            self.validation_errors = ValidationErrors::new();
        }
    }
}

pub struct NewCostingTab {
    form: FormData,
    props: Props,
    currencies: Vec<CommodityType>,
    link: ComponentLink<Self>,
}

pub enum Msg {
    ChangeName(String),
    ChangeWorkingCurrency(CommodityType),
    Create,
    Cancel,
}

#[derive(Clone, Properties, PartialEq)]
pub struct Props {
    pub lang: unic_langid::LanguageIdentifier,
    pub router: AppRouterRef,
}

impl Component for NewCostingTab {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let mut currencies = commodity::all_iso4217_currencies();
        currencies.sort_by(|a, b| a.id.cmp(&b.id));

        NewCostingTab {
            form: FormData::new(),
            props,
            currencies,
            link,
        }
    }

    fn update(&mut self, msg: Msg) -> ShouldRender {
        match msg {
            Msg::ChangeName(name) => {
                self.form.name = name;
                self.form.run_validation();
            }
            Msg::ChangeWorkingCurrency(working_currency) => {
                self.form.working_currency = working_currency;
                self.form.run_validation();
            }
            Msg::Create => {
                self.form.run_validation();
            }
            Msg::Cancel => {
                self.props.router.borrow_mut().set_route(AppRoute::Index);
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

        let onchange_working_currency = self.link.callback(Msg::ChangeWorkingCurrency);
        let onchange_name = self.link.callback(|data: ChangeData| {
            match data {
                ChangeData::Value(value) => Msg::ChangeName(value),
                _ => panic!("invalid data type"),
            }
        });
        let onclick_cancel = self.link.callback(|_| Msg::Cancel);
        let onclick_create = self.link.callback(|_| Msg::Create);

        let mut name_classes = vec!["input".to_string()];
        let mut name_validation_error = html! {};
        if let Some(errors) = self.form.validation_errors.field_errors().get("name") {
            name_classes.push("is-danger".to_string());
            let error_message = validation_messages(errors);
            name_validation_error = html!{<p class="help is-danger">{ error_message }</p>}
        }

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
                    // <form>
                        <div class="field">
                            <label class="label">{ tr!("Name") }</label>
                            <div class="control">
                                <input class=name_classes type="text" placeholder=tr!("Tab Name") onchange=onchange_name/>
                            </div>
                            { name_validation_error }
                        </div>
                        <div class="field">
                            <label class="label">{ tr!("Working Currency") }</label>
                            <div class="control">
                                <Select<CommodityType>
                                        selected=self.form.working_currency.clone()
                                        options=self.currencies.clone()
                                        onchange=onchange_working_currency
                                        />
                            </div>
                        </div>

                        <div class="field is-grouped">
                            <div class="control">
                                <button 
                                    class="button is-link" 
                                    onclick=onclick_create 
                                    disabled=!self.form.validation_errors.is_empty()>
                                    { tr!("Create") }
                                </button> 
                            </div>
                            <div class="control">
                                <button class="button is-link is-light" onclick=onclick_cancel>{ tr!("Cancel") }</button>
                            </div>
                        </div>
                    // </form>
                </div>
            </>
        }
    }
    fn destroy(&mut self) {}
}
