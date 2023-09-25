#![allow(unused_braces)]
#![allow(non_snake_case)]

use crate::app::{remove_counter, remove_phase, SerCounter, SessionUser};

use super::*;
use leptos::*;
use leptos_router::A;
use web_sys::MouseEvent;

#[component]
pub fn CountableContextMenu(
    cx: Scope,
    show_overlay: RwSignal<bool>,
    location: ReadSignal<(i32, i32)>,
    countable_id: i32,
    is_phase: bool,
) -> impl IntoView {
    let edit_location = format!(
        "/edit/{}/{}",
        if is_phase { "phase" } else { "counter" },
        countable_id
    );

    let on_del_click = move |ev: MouseEvent| {
        ev.stop_propagation();
        if let Some(_) = use_context::<RwSignal<Option<SessionUser>>>(cx)
            .map(|s| s.get_untracked())
            .flatten()
        {
            let user = expect_context::<RwSignal<Option<SessionUser>>>(cx);
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
                create_effect(cx, move |_| {
                    expect_context::<Resource<Option<SessionUser>, Vec<SerCounter>>>(cx).refetch();
                })
            });
        }
    };

    view! { cx,
        <Overlay show_overlay=show_overlay location=location>
            <ContextMenuNav href=edit_location.clone()>
                <span>Edit</span>
            </ContextMenuNav>
            <ContextMenuRow on:click=on_del_click>
                <span>Delete</span>
            </ContextMenuRow>
        </Overlay>
    }
}

#[component]
pub fn ContextMenuRow(cx: Scope, children: Children) -> impl IntoView {
    view! { cx,
        <div class="context-menu-row">{ children(cx) }</div>
    }
}

#[component]
pub fn ContextMenuNav(cx: Scope, href: String, children: Children) -> impl IntoView {
    let on_click = move |_| {
        use_context::<RwSignal<CloseOverlays>>(cx).map(|t| t.update(|_| ()));
    };

    view! { cx,
        <A href=href class="remove-underline" on:click=on_click><div class="context-menu-row">{ children(cx) }</div></A>
    }
}
