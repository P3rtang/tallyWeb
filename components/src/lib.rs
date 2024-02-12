#![feature(let_chains)]

mod about;
mod account_icon;
mod loading_screen;
mod message;
mod saving_screen;
mod sidebar;
mod slider;
mod spinner;
mod treeview;
pub use about::*;
pub use account_icon::*;
pub use loading_screen::*;
pub use message::*;
pub use saving_screen::*;
pub use sidebar::*;
pub use slider::*;
pub use spinner::*;
pub use treeview::*;

use leptos::{logging::warn, *};

#[derive(Debug, Clone)]
pub struct CloseOverlays();

#[component]
pub fn Overlay(
    show_overlay: RwSignal<bool>,
    location: ReadSignal<(i32, i32)>,
    children: ChildrenFn,
    #[prop(optional)] accent_color: Option<Signal<String>>,
) -> impl IntoView
where {
    if let Some(close_signal) = use_context::<RwSignal<CloseOverlays>>() {
        create_effect(move |_| {
            close_signal.track();
            show_overlay.set(false);
        });
    } else {
        warn!("No `close overlay` signal available");
    }

    let border_style = move || {
        format!(
            "border: 2px solid {};",
            accent_color.map(|ac| ac()).unwrap_or_default()
        )
    };

    let location_style = move || {
        format!(
            "left: {}px; top: {}px;",
            location().0 + 10,
            location().1 + 10
        )
    };

    view! {
        <Show when=move || { show_overlay.get() } fallback=|| ()>
            <div class="overlay" style=border_style() + &location_style()>
                {children()}
            </div>
        </Show>
    }
}
