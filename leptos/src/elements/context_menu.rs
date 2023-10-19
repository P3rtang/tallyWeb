#![allow(unused_braces)]
#![allow(non_snake_case)]

use crate::{
    app::{remove_counter, remove_phase, SessionUser},
    countable::SerCounter,
};

use components::{CloseOverlays, Overlay};
use leptos::*;
use leptos_router::A;
use web_sys::MouseEvent;

#[component]
pub fn CountableContextMenu(
    show_overlay: RwSignal<bool>,
    location: ReadSignal<(i32, i32)>,
    counters_resource: Resource<Option<SessionUser>, Vec<SerCounter>>,
    countable_id: i32,
    is_phase: bool,
    #[prop(optional)] accent_color: Option<Signal<String>>,
) -> impl IntoView {
    let (edit_location, _) = create_signal(format!(
        "/edit/{}/{}",
        if is_phase { "phase" } else { "counter" },
        countable_id
    ));

    let on_del_click = move |ev: MouseEvent| {
        ev.stop_propagation();
        if use_context::<Memo<Option<SessionUser>>>()
            .and_then(|s| s.get_untracked()).is_some()
        {
            let user = expect_context::<Memo<Option<SessionUser>>>();
            spawn_local(async move {
                if is_phase {
                    let _ = remove_phase(
                        user.get_untracked().unwrap().username,
                        user.get_untracked().unwrap().token,
                        countable_id,
                    )
                    .await;
                } else {
                    let _ = remove_counter(
                        user.get_untracked().unwrap().username,
                        user.get_untracked().unwrap().token,
                        countable_id,
                    )
                    .await;
                }
                create_effect(move |_| counters_resource.refetch());
            });
        }
    };

    view! {
        <Show
            when=move || accent_color.is_some()
            fallback=move || view! {
                <Overlay show_overlay=show_overlay location=location>
                    <ContextMenuNav href=edit_location.get()>
                        <span>Edit</span>
                    </ContextMenuNav>
                    <ContextMenuRow on:click=on_del_click>
                        <span>Delete</span>
                    </ContextMenuRow>
                </Overlay>
            }
        >
            <Overlay show_overlay=show_overlay location=location accent_color=accent_color.unwrap()>
                <ContextMenuNav href=edit_location.get()>
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
pub fn ContextMenuNav(href: String, children: Children) -> impl IntoView {
    let on_click = move |_| {
        if let Some(t) = use_context::<RwSignal<CloseOverlays>>() { t.update(|_| ()) }
    };

    view! {
        <A href=href class="remove-underline" on:click=on_click>
            <div class="context-menu-row">{ children() }</div>
        </A>
    }
}
