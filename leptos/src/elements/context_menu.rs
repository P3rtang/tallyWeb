#![allow(unused_braces)]
#![allow(non_snake_case)]

use leptos::*;
use leptos_router::{A, ToHref};
use web_sys::MouseEvent;

use components::{CloseOverlays, Overlay};

#[component]
pub fn CountableContextMenu(
    show_overlay: RwSignal<bool>,
    location: ReadSignal<(i32, i32)>,
    #[prop(into)] key: Signal<uuid::Uuid>,
    #[prop(optional)] accent_color: Option<Signal<String>>,
) -> impl IntoView {
    let on_del_click = move |ev: MouseEvent| {
        ev.stop_propagation();
        spawn_local(async move {});
    };

    view! {
        <Show
            when=move || accent_color.is_some()
            fallback=move || view! {
                <Overlay show_overlay=show_overlay location=location>
                    <ContextMenuNav href=move || format!("edit/{}", key().to_string())>
                        <span>Edit</span>
                    </ContextMenuNav>
                    <ContextMenuRow on:click=on_del_click>
                        <span>Delete</span>
                    </ContextMenuRow>
                </Overlay>
            }
        >
            <Overlay show_overlay=show_overlay location=location accent_color=accent_color.unwrap()>
                <ContextMenuNav href=move || format!("edit/{}", key().to_string())>
                    <span>Edit</span>
                </ContextMenuNav>
                <ContextMenuRow on:click=on_del_click>
                    <span>Delete</span>
                </ContextMenuRow>
            </Overlay>
        </Show>
    }
}

#[component]
pub fn ContextMenuRow(children: Children) -> impl IntoView {
    view! {
        <div class="context-menu-row">{ children() }</div>
    }
}

#[component]
pub fn ContextMenuNav<H>(href: H, children: Children) -> impl IntoView
where H: ToHref + 'static,
{
    let on_click = move |_| {
        if let Some(t) = use_context::<RwSignal<CloseOverlays>>() {
            t.update(|_| ())
        }
    };

    view! {
        <A href=href class="remove-underline" on:click=on_click>
            <div class="context-menu-row">{ children() }</div>
        </A>
    }
}
