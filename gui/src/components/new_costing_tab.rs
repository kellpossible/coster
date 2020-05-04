use crate::bulma::components::Icon;
use crate::bulma::form::form::{self, Form, FormFieldLink};
use crate::bulma::{components::SelectField, FieldKey};
use crate::validation::{Validatable, Validated, ValidationError, ValidationErrors, Validator};
use crate::{bulma::components::Select, AppRoute, AppRouterRef};
use commodity::CommodityType;
use std::fmt::Display;
use tr::tr;
use yew::{html, ChangeData, Children, Component, ComponentLink, Html, Properties, ShouldRender};

#[derive(PartialEq, Clone, Copy, Hash, Eq)]
enum FormFields {
    Form,
    Name,
    WorkingCurrency,
    Participant(u32),
}

impl FieldKey for FormFields {
    fn field_label(&self) -> String {
        match self {
            FormFields::Form => "Form".to_string(),
            FormFields::Name => tr!("Tab Name"),
            FormFields::WorkingCurrency => tr!("Working Currency"),
            FormFields::Participant(n) => tr!("Particapant {0}", n),
        }
    }
}

impl Display for FormFields {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.field_label())
    }
}

pub struct FormData {
    name: String,
    working_currency: Option<CommodityType>,
}

impl FormData {
    pub fn new() -> Self {
        Self {
            name: "".to_string(),
            working_currency: None,
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

    fn select_field<T, F>(
        &self,
        field: FormFields,
        selected: Option<T>,
        options: Vec<T>,
        onchange: F,
    ) -> Html
    where
        T: ToString + PartialEq + Clone + 'static,
        F: Fn(T) -> Msg + 'static,
    {
        let onchange_callback = self.link.callback(onchange);

        let mut classes = vec![];
        let validation_error = if let Some(errors) = self.validation_errors.get(&field) {
            classes.push("is-danger".to_string());
            let error_message = errors.to_string();
            html! {<p class="help is-danger">{ error_message }</p>}
        } else {
            html! {}
        };

        html! {
            <div class="field">
                <label class="label">{ field.field_label() }</label>
                <div class="control">
                    <Select<T>
                            selected=selected
                            options=options
                            div_classes=classes
                            onchange=onchange_callback
                            />
                </div>
                { validation_error }
            </div>
        }
    }

    fn input_field<F>(&self, field: FormFields, placeholder: String, onchange: F) -> Html
    where
        F: Fn(String) -> Msg + 'static,
    {
        let mut classes = vec!["input".to_string()];
        let mut validation_error = html! {};
        if let Some(errors) = self.validation_errors.get(&field) {
            classes.push("is-danger".to_string());
            let error_message = errors.to_string();
            validation_error = html! {<p class="help is-danger">{ error_message }</p>}
        }

        let onchange_callback = self.link.callback(move |data: ChangeData| match data {
            ChangeData::Value(value) => (onchange)(value),
            _ => panic!("invalid data type"),
        });

        html! {
            <div class="field">
                <label class="label">{ field.field_label() }</label>
                <div class="control">
                    <input
                        class=classes
                        type="text"
                        placeholder=placeholder
                        onchange=onchange_callback/>
                </div>
                { validation_error }
            </div>
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

        let form = Validated::new(
            FormData::new(),
            FormFields::Form,
            Validator::new()
                .validation(|form: &FormData, _| {
                    if form.name.trim().len() == 0 {
                        Err(ValidationError::new(FormFields::Name)
                            .with_message(|_| tr!("This field cannot be empty")))
                    } else {
                        Ok(())
                    }
                })
                .validation(|form: &FormData, _| {
                    if form.working_currency.is_none() {
                        Err(ValidationError::new(FormFields::WorkingCurrency)
                            .with_message(|_| tr!("Please select a working currency")))
                    } else {
                        Ok(())
                    }
                }),
        );

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
                self.form.value.working_currency = Some(working_currency);
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
        let onclick_cancel = self.link.callback(|_| Msg::Cancel);
        let onclick_create = self.link.callback(|_| Msg::Create);

        let working_currency_validator: Validator<Option<CommodityType>, FormFields> =
            Validator::new().validation(|working_currency: &Option<CommodityType>, _| {
                if working_currency.is_none() {
                    Err(ValidationError::new(FormFields::WorkingCurrency)
                        .with_message(|_| tr!("Please select a working currency")))
                } else {
                    Ok(())
                }
            });

        // let name_field =
        //     self.input_field(FormFields::Name, tr!("Participant name"), Msg::ChangeName);

        let form_field_link: FormFieldLink<FormFields> = FormFieldLink::new();

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
                        <Form<FormFields> field_link = form_field_link.clone()>
                            <SelectField<CommodityType, FormFields>
                                field_key = FormFields::WorkingCurrency
                                options = self.currencies.clone()
                                validator = working_currency_validator
                                form_link = form_field_link
                                />
                        </Form<FormFields>>
                    // </form>
                </div>
            </>
        }
    }
    fn destroy(&mut self) {}
}
