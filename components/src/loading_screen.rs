#![allow(non_snake_case)]
use super::*;
use leptos::*;

#[component]
pub fn LoadingMessage() -> impl IntoView {
    view! {
        <div style="display: flex; align-items: center;">
            <Spinner/>
            <b style="font-size: 20px; padding-left: 24px;">Loading</b>
        </div>
    }
}

#[component]
pub fn LoadingScreen(#[prop(optional)] accent_color: Option<Signal<String>>) -> impl IntoView {
    let border_style = move || {
        format!(
            "2px solid {};",
            accent_color
                .map(|ac| ac())
                .unwrap_or(String::from("#ffe135"))
        )
    };

    view! {
        <div class="notification-box" style:border=border_style>
            <LoadingMessage/>
        </div>
    }
}
