#![allow(unused_braces)]

use crate::app::{navigate, remove_counter, remove_phase, SerCounter, SessionUser};

use super::*;
use leptos::*;
use web_sys::MouseEvent;

#[component]
pub fn CountableContextMenu(
    cx: Scope,
    show_overlay: RwSignal<bool>,
    location: ReadSignal<(i32, i32)>,
    countable_id: i32,
    is_phase: bool,
) -> impl IntoView {
    let on_edit_click = move |ev: MouseEvent| {
        ev.stop_propagation();
        navigate(cx, format!("/edit/{countable_id}"));
    };

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
            <ContextMenuRow on:click=on_edit_click>
                <span>Edit</span>
            </ContextMenuRow>
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
