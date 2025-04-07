use leptos::{
    either::Either,
    ev, html,
    prelude::*,
    tachys::renderer::{dom::Element, RemoveEventHandler},
};
use wasm_bindgen::JsCast;

#[component]
pub fn ToolTip(
    #[prop(into)] tooltip: Signal<String>,
    #[prop(optional, default=std::time::Duration::from_secs(2))] delay: std::time::Duration,
    children: ChildrenFn,
) -> impl IntoView {
    let is_shown = RwSignal::new(false);
    let is_hovering = RwSignal::new(false);
    let mouse_pos = RwSignal::new((0, 0));

    let handle_mouse_over = move |_: ev::MouseEvent| {
        is_hovering.try_set(true);
        set_timeout(
            move || {
                if is_hovering.try_get().unwrap_or_default() {
                    is_shown.try_set(true);
                }
            },
            delay,
        )
    };

    let handle_mouse_out = move |_: ev::MouseEvent| {
        is_hovering.try_set(false);
        is_shown.try_set(false);
    };

    let handle_mouse_move = move |ev: ev::MouseEvent| {
        if !is_shown() {
            mouse_pos.try_set((ev.x(), ev.y()));
        }
    };

    // TODO: do the attributes need to be captured ???
    view! {
        <Show when=is_shown>
            <tool-tip
                style:z-index="100"
                style:position="absolute"
                style:padding="8px"
                style:left=move || format!("{}px", mouse_pos().0 + 12)
                style:top=move || format!("{}px", mouse_pos().1 + 20)
            >
                {tooltip}
            </tool-tip>
        </Show>
        <div
            on:mouseover=handle_mouse_over
            on:mouseout=handle_mouse_out
            on:mousemove=handle_mouse_move
        >
            {children()}
        </div>
    }
}

pub fn with_tooltip<VF, IV>(wrapped: VF, tooltip: Signal<Option<String>>) -> impl IntoView
where
    VF: Fn() -> IV + Clone + Send + Sync + 'static,
    IV: IntoView + 'static,
{
    view! {
        {match tooltip.get() {
            Some(tip) => Either::Left(view! { <ToolTip tooltip=tip>{wrapped()}</ToolTip> }),
            None => Either::Right(wrapped),
        }}
    }
}
