use yew::{html, Component, ComponentLink, Html, ShouldRender};
use tr::tr;
use web_sys::console;
use log::debug;

pub struct ClickerButton {
    link: ComponentLink<Self>,
}

pub enum Msg {
    Click,
}

impl Component for ClickerButton {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        ClickerButton { link }
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
}