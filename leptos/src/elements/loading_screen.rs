#![allow(non_snake_case)]
use leptos::*;

#[component]
pub fn LoadingScreen() -> impl IntoView {
    view! {
        <p class="notification-box" style="border: 2px solid #ffe135">Loading...</p>
    }
}
