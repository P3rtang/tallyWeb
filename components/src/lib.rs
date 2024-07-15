#![feature(let_chains)]

mod loading_screen;
mod message;
mod progressbar;
mod saving_screen;
mod select;
mod sidebar;
mod slider;
mod spinner;
mod tooltip;
mod treeview;

pub use loading_screen::*;
pub use message::*;
pub use progressbar::*;
pub use saving_screen::*;
pub use select::{Select, SelectOption};
pub use sidebar::*;
pub use slider::*;
pub use spinner::*;
pub use tooltip::*;
pub use treeview::*;

use leptos::{logging::warn, *};

#[derive(Debug, Clone)]
pub struct CloseOverlays();

#[component]
pub fn Overlay(
    #[prop(attrs)] attrs: Vec<(&'static str, Attribute)>,
    show_overlay: RwSignal<bool>,
    location: ReadSignal<(i32, i32)>,
    children: ChildrenFn,
) -> impl IntoView {
    if let Some(close_signal) = use_context::<RwSignal<CloseOverlays>>() {
        create_effect(move |_| {
            close_signal.track();
            show_overlay.set(false);
        });
    } else {
        warn!("No `close overlay` signal available");
    }

    let location_style = move || {
        format!(
            "left: {}px; top: {}px;",
            location().0 + 10,
            location().1 + 10
        )
    };

    view! {
        <Show when=move || { show_overlay.get() } fallback=|| ()>
            <div style=location_style() {..attrs.clone()}>
                {children()}
            </div>
        </Show>
    }
}
