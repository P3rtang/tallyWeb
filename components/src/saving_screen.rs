use super::*;
use leptos::*;

#[component]
pub fn SavingMessage() -> impl IntoView {
    view! {
        <div style="display: flex; align-items: center;">
            <Spinner/>
            <b style="font-size: 20px; padding-left: 24px;">Saving</b>
        </div>
    }
}

#[component]
pub fn SavingScreen(#[prop(optional)] accent_color: Option<Signal<String>>) -> impl IntoView {
    let border_style = move || {
        format!(
            "2px solid {};",
            accent_color.map(|ac| ac()).unwrap_or(String::new())
        )
    };

    view! {
        <div class="notification-box" style:border=border_style>
            <SavingMessage/>
        </div>
    }
}
