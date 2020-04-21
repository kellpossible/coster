//! This module contains implementation of `Select` component.
//! You can use it instead `<select>` tag, because the component
//! helps you to track selected value in an original type. Example:
//! 
//! Ripped out of Yew source code https://github.com/yewstack/yew/blob/8edf136da6ba1955c847c5860ec55623a27c08e9/src/components/select.rs
//! License for original code: https://github.com/yewstack/yew/blob/master/LICENSE-APACHE 
//! Modified to support css `bulma` classes.
//!
//! ```
//!# use yew::{Html, Component, components::Select, ComponentLink, html};
//! #[derive(PartialEq, Clone)]
//! enum Scene {
//!     First,
//!     Second,
//! }
//!# struct Model { link: ComponentLink<Self> };
//!# impl Component for Model {
//!#     type Message = ();type Properties = ();
//!#     fn create(props: Self::Properties,link: ComponentLink<Self>) -> Self {unimplemented!()}
//!#     fn update(&mut self,msg: Self::Message) -> bool {unimplemented!()}
//!#     fn change(&mut self, _: Self::Properties) -> bool {unimplemented!()}
//!#     fn view(&self) -> Html {unimplemented!()}}
//! impl ToString for Scene {
//!     fn to_string(&self) -> String {
//!         match self {
//!             Scene::First => "First".to_string(),
//!             Scene::Second => "Second".to_string()
//!         }
//!     }
//! }
//!
//! fn view(link: ComponentLink<Model>) -> Html {
//!     let scenes = vec![Scene::First, Scene::Second];
//!     html! {
//!         <Select<Scene> options=scenes onchange=link.callback(|_| ()) />
//!     }
//! }
//! ```

use crate::bulma::{Size, components::{icon, Icon}};
use yew::callback::Callback;
use yew::html::{ChangeData, Component, ComponentLink, Html, NodeRef, ShouldRender};
use yew::macros::{html, Properties};
use web_sys::HtmlSelectElement;
use log::debug;


/// `Select` component.
#[derive(Debug)]
pub struct Select<T: ToString + PartialEq + Clone + 'static> {
    props: Props<T>,
    select_ref: NodeRef,
    link: ComponentLink<Self>,
}

/// Internal message of the component.
#[derive(Debug)]
pub enum Msg {
    /// This message indicates the option with id selected.
    Selected(Option<usize>),
}

/// Properties of `Select` component.
#[derive(PartialEq, Clone, Properties, Debug)]
pub struct Props<T: Clone> {
    /// Initially selected value.
    #[prop_or_default]
    pub selected: Option<T>,
    /// Disabled the component's selector.
    #[prop_or_default]
    pub disabled: bool,
    /// Options are available to choose.
    #[prop_or_default]
    pub options: Vec<T>,
    #[prop_or_default]
    pub div_classes: Vec<String>,
    #[prop_or_default]
    pub icon_props: Option<icon::Props>,
    #[prop_or_default]
    pub size: Size,
    /// Callback to handle changes.
    pub onchange: Callback<T>,
}

impl<T> Component for Select<T>
where
    T: ToString + PartialEq + Clone + 'static,
{
    type Message = Msg;
    type Properties = Props<T>;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            props,
            select_ref: NodeRef::default(),
            link,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Selected(value) => {
                if let Some(idx) = value {
                    let item = self.props.options.get(idx - 1).cloned();
                    if let Some(value) = item {
                        self.props.onchange.emit(value);
                    }
                }
            }
        }
        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        if self.props.selected != props.selected {
            if let Some(select) = self.select_ref.cast::<HtmlSelectElement>() {
                let val = props
                    .selected
                    .as_ref()
                    .map(|v| v.to_string())
                    .unwrap_or_default();
                
                select.set_value(&val)
            }
        }
        self.props = props;
        true
    }

    fn view(&self) -> Html {
        let selected = self.props.selected.as_ref();
        let view_option = |value: &T| {
            let flag = selected == Some(value);
            html! {
                <option value=value.to_string() selected=flag>{ value.to_string() }</option>
            }
        };

        let mut div_classes = vec!["select".to_string()];
        
        let size_class_vec = match self.props.size.to_class() {
            Some(size) => vec![size],
            None => vec![],
        };

        div_classes.extend(size_class_vec.clone());

        div_classes.extend(self.props.div_classes.clone());

        let inner = html! {
            <div class=div_classes>
                <select ref=self.select_ref.clone() disabled=self.props.disabled onchange=self.onchange()>
                    <option value="" disabled=true selected=selected.is_none()>
                        { "↪" }
                    </option>
                    { for self.props.options.iter().map(view_option) }
                </select>
            </div>
        };

        if self.props.icon_props.is_some() {
            let mut icon_props = self.props.icon_props.as_ref().unwrap().clone();
            icon_props.span_class.push("is-left".to_string());

            html! {
                <div class="control has-icons-left">
                {
                    inner
                }
                <Icon with icon_props/>
                </div>
            }
        }
        else {
            inner
        }
    }
}

impl<T> Select<T>
where
    T: ToString + PartialEq + Clone + 'static,
{
    fn onchange(&self) -> Callback<ChangeData> {
        self.link.callback(|event| match event {
            ChangeData::Select(elem) => {
                let value = elem.selected_index();
                let value = Some(value as usize);
                Msg::Selected(value)
            }
            _ => {
                unreachable!();
            }
        })
    }
}