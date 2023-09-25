#![allow(non_snake_case)]
use leptos::*;

#[component]
pub fn LoadingScreen(cx: Scope) -> impl IntoView {
    view! { cx,
        <p class="notification-box" style="border: 2px solid #ffe135">Loading...</p>
    }
}
