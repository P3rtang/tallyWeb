use super::*;
use leptos::*;

#[component]
pub fn SavingScreen() -> impl IntoView {
    view! {
        <div style="display: flex; align-items: center;">
            <Spinner/>
            <b style="font-size: 20px; padding-left: 24px;">Saving</b>
        </div>
    }
}
