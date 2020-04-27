use yew::{html, Html};

pub fn centered(inner: Html) -> Html {
    html! {
        <div class="section">
            <div class="columns">
                <div class="column is-one-quarter is-desktop"></div>
                <div class="column">
                    <div class="container">
                        { inner }
                    </div>
                </div>
                <div class="column is-one-quarter is-desktop"></div>
            </div>
        </div>
    }
}
