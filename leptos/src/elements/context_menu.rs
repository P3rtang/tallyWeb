#![allow(unused_braces)]
#![allow(non_snake_case)]

use crate::{
    app::{remove_counter, remove_phase, SelectionSignal, SessionUser},
    countable::{CountableKind, SerCounter},
};

use components::{CloseOverlays, Message, Overlay};
use leptos::*;
use leptos_router::A;
use web_sys::MouseEvent;

#[component]
pub fn CountableContextMenu(
    show_overlay: RwSignal<bool>,
    location: ReadSignal<(i32, i32)>,
    #[prop(into)] key: Signal<String>,
    #[prop(optional)] accent_color: Option<Signal<String>>,
) -> impl IntoView {
    let message = expect_context::<Message>();
    let data = expect_context::<Resource<Option<SessionUser>, Vec<SerCounter>>>();

    let countable_kind = move || CountableKind::try_from(key.get_untracked());

    let (edit_location, _) = create_signal(if let Ok(kind) = countable_kind() {
        format!(
            "/edit/{}/{}",
            if key.get_untracked().starts_with('p') {
                "phase"
            } else {
                "counter"
            },
            kind.id()
        )
    } else {
        "/edit".to_string()
    });

    let on_del_click = move |ev: MouseEvent| {
        ev.stop_propagation();
        let user = expect_context::<Memo<Option<SessionUser>>>();
        spawn_local(async move {
            match countable_kind() {
                Ok(CountableKind::Counter(id)) => {
                    if remove_counter(
                        user.get_untracked().unwrap().username,
                        user.get_untracked().unwrap().token,
                        id,
                    )
                    .await
                    .is_ok()
                    {
                        data.refetch()
                    }
                }
                Ok(CountableKind::Phase(id)) => {
                    if remove_phase(
                        user.get_untracked().unwrap().username,
                        user.get_untracked().unwrap().token,
                        id,
                    )
                    .await
                    .is_ok()
                    {
                        data.refetch()
                    }
                }
                Err(_) => message.set_err("Could not convert counter Id"),
            }
        });
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
