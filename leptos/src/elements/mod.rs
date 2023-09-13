pub mod account_icon;
pub mod context_menu;
pub use account_icon::*;
pub use context_menu::*;

use crate::app::AccountAccentColor;
use leptos::*;

#[derive(Debug, Clone)]
pub struct CloseOverlays();

impl CloseOverlays {
    pub fn new() -> Self {
        Self()
    }
}

#[component]
pub fn Overlay(
    cx: Scope,
    show_overlay: RwSignal<bool>,
    location: ReadSignal<(i32, i32)>,
    children: ChildrenFn,
) -> impl IntoView
where {
    if let Some(close_signal) = use_context::<RwSignal<CloseOverlays>>(cx) {
        create_effect(cx, move |_| {
            close_signal.get();
            show_overlay.set(false);
        });
    } else {
        debug_warn!("No `close overlay` signal available");
    }

    let accent_color = expect_context::<Signal<AccountAccentColor>>(cx);
    let border_style = move || format!("border: 2px solid {};", accent_color.get());
    let location_style = move || {
        format!(
            "left: {}px; top: {}px;",
            location().0 + 10,
            location().1 + 10
        )
    };

    view! { cx,
        <Show
            when=move || { show_overlay.get() }
            fallback=|_| ()
        >
            <div
                class="overlay"
                style={ border_style() + &location_style() }
            >{ children(cx) }</div>
        </Show>
    }
}
