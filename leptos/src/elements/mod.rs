mod about;
mod account_icon;
mod context_menu;
mod infobox;
mod loading_screen;
mod message;
mod navbar;
mod sidebar;
mod sort_search;
mod treeview;
pub use about::*;
pub use account_icon::*;
pub use context_menu::*;
pub use infobox::*;
pub use loading_screen::*;
pub use message::*;
pub use navbar::*;
pub use sidebar::*;
pub use sort_search::*;
pub use treeview::*;

use leptos::{logging::warn, *};

#[derive(Debug, Clone)]
pub struct CloseOverlays();

#[component]
pub fn Overlay(
    show_overlay: RwSignal<bool>,
    location: ReadSignal<(i32, i32)>,
    children: ChildrenFn,
) -> impl IntoView
where {
    if let Some(close_signal) = use_context::<RwSignal<CloseOverlays>>() {
        create_effect(move |_| {
            close_signal.get();
            show_overlay.set(false);
        });
    } else {
        warn!("No `close overlay` signal available");
    }

    let preferences = expect_context::<RwSignal<crate::app::Preferences>>();
    let border_style = create_read_slice(preferences, |pref| {
        format!("border: 2px solid {};", pref.accent_color.0)
    });

    let location_style = move || {
        format!(
            "left: {}px; top: {}px;",
            location().0 + 10,
            location().1 + 10
        )
    };

    view! {
        <Show
            when=move || { show_overlay.get() }
            fallback=|| ()
        >
            <div
                class="overlay"
                style={ border_style() + &location_style() }
            >{ children() }</div>
        </Show>
    }
}
