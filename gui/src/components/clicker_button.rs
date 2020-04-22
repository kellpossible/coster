use yew::{html, Component, ComponentLink, Html, ShouldRender, Properties};
use tr::tr;
use web_sys::console;
use log::debug;

pub struct ClickerButton {
    props: Props,
    link: ComponentLink<Self>,
}

#[derive(Clone, Properties, PartialEq)]
pub struct Props {
    pub lang: unic_langid::LanguageIdentifier
}

pub enum Msg {
    Click,
}

impl Component for ClickerButton {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        ClickerButton { props, link }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Click => {
                console::log_1(&tr!("Hello World, this is me!").into());
            }
        }
        true
    }

    fn view(&self) -> Html {
        debug!("Rendering Clicker Button");
        html! {
            <div>
                <button class="button" onclick=self.link.callback(|_| Msg::Click)>{ tr!("Click") }</button>
            </div>
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
}