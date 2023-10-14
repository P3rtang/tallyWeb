use components::{AccountIcon, ShowSidebar};
use leptos::*;
use leptos_router::A;

use crate::app::{Preferences, SessionUser};

#[component]
pub fn Navbar() -> impl IntoView {
    let user = expect_context::<Memo<Option<SessionUser>>>();
    let preferences = expect_context::<RwSignal<Preferences>>();
    let accent_color = create_read_slice(preferences, |pref| pref.accent_color.0.clone());

    let show_sidebar = expect_context::<RwSignal<ShowSidebar>>();
    let toggle_sidebar = move |_| show_sidebar.update(|s| s.0 = !s.0);

    view! {
        <nav>
            <button id="toggle-sidebar" on:click=toggle_sidebar>
                <i class="fa-solid fa-bars"></i>
            </button>
            <A href="/"><img src="/favicon.svg" width=48 height=48 alt="Home" class="tooltip-parent"/>
                <span class="tooltip bottom">Home</span>
            </A>
            <Show
                when=move || user().is_some()
                fallback=|| view! { <div id="user-icon" style="background: #555555;"/> }
            >
                <AccountIcon username=move || user.get().unwrap_or_default().username accent_color/>
            </Show>
        </nav>
    }
}
