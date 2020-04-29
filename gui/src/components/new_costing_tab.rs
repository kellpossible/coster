use crate::validation::{Validatable, Validated, ValidationError, ValidationErrors};
use crate::{bulma::components::Select, AppRoute, AppRouterRef};
use commodity::CommodityType;
use log::info;
use tr::tr;
use yew::{html, ChangeData, Component, ComponentLink, Html, Properties, ShouldRender};

#[derive(PartialEq, Clone)]
enum FormFields {
    Form,
    Name,
}

pub struct FormData {
    name: String,
    working_currency: CommodityType,
}

impl FormData {
    pub fn new() -> Self {
        Self {
            name: "".to_string(),
            working_currency: CommodityType::from_currency_alpha3("AUD").unwrap(),
        }
    }
}

pub struct NewCostingTab {
    form: Validated<FormData, FormFields>,
    validation_errors: ValidationErrors<FormFields>,
    props: Props,
    currencies: Vec<CommodityType>,
    link: ComponentLink<Self>,
}

impl NewCostingTab {
    fn validate_form(&mut self) {
        self.validation_errors = match self.form.validate() {
            Ok(()) => ValidationErrors::default(),
            Err(errors) => errors,
        }
    }
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

        let form = Validated::new(FormData::new(), FormFields::Form).validator(|form, _| {
            if form.name.trim().len() == 0 {
                Err(ValidationError::new(FormFields::Name).with_message(|_| tr!("Cannot be empty")))
            } else {
                Ok(())
            }
        });

        NewCostingTab {
            form,
            validation_errors: ValidationErrors::default(),
            props,
            currencies,
            link,
        }
    }

    fn update(&mut self, msg: Msg) -> ShouldRender {
        match msg {
            Msg::ChangeName(name) => {
                self.form.value.name = name.trim().to_string();
                self.validate_form();
            }
            Msg::ChangeWorkingCurrency(working_currency) => {
                self.form.value.working_currency = working_currency;
                self.validate_form();
            }
            Msg::Create => {
                self.validate_form();
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
        let onchange_name = self.link.callback(|data: ChangeData| match data {
            ChangeData::Value(value) => Msg::ChangeName(value),
            _ => panic!("invalid data type"),
        });
        let onclick_cancel = self.link.callback(|_| Msg::Cancel);
        let onclick_create = self.link.callback(|_| Msg::Create);

        let mut name_classes = vec!["input".to_string()];
        let mut name_validation_error = html! {};
        if let Some(errors) = self.validation_errors.get(FormFields::Name) {
            name_classes.push("is-danger".to_string());
            let error_message = errors.to_string();
            name_validation_error = html! {<p class="help is-danger">{ error_message }</p>}
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
                                        selected=self.form.value.working_currency.clone()
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
                                    disabled=!self.validation_errors.is_empty()>
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
