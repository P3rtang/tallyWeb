#![allow(unused_braces)]

use crate::app::{navigate, remove_counter, SerCounter, SessionUser};

use super::*;
use leptos::*;
use web_sys::MouseEvent;

#[component]
pub fn CounterContextMenu(
    cx: Scope,
    show_overlay: RwSignal<bool>,
    location: ReadSignal<(i32, i32)>,
    counter_id: i32,
) -> impl IntoView {
    let on_edit_click = move |ev: MouseEvent| {
        ev.stop_propagation();
        navigate(cx, format!("/edit/{counter_id}"));
    };

    let on_del_click = move |ev: MouseEvent| {
        ev.stop_propagation();
        if let Some(_) = use_context::<RwSignal<Option<SessionUser>>>(cx)
            .map(|s| s.get_untracked())
            .flatten()
        {
            let user = expect_context::<RwSignal<Option<SessionUser>>>(cx);
            spawn_local(async move {
                let _ = remove_counter(
                    user.get_untracked().unwrap().username,
                    user.get_untracked().unwrap().token,
                    counter_id,
                )
                .await;
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
