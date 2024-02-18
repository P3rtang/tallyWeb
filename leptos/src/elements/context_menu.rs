#![allow(unused_braces)]
#![allow(non_snake_case)]

use leptos::*;
use leptos_router::{ToHref, A};

use super::*;
use components::{CloseOverlays, Overlay};

#[component]
pub fn CountableContextMenu(
    show_overlay: RwSignal<bool>,
    location: ReadSignal<(i32, i32)>,
    #[prop(into)] key: Signal<uuid::Uuid>,
    #[prop(optional)] accent_color: Option<Signal<String>>,
) -> impl IntoView {
    let user = expect_context::<RwSignal<UserSession>>();
    let state_rsrc = expect_context::<StateResource>();
    let selection = expect_context::<SelectionSignal>();
    let save_handler = expect_context::<SaveHandlerCountable>();
    let delete_action = create_server_action::<api::RemoveCountable>();

    let is_success = create_read_slice(selection, move |sel| {
        sel.get(&key.get_untracked())
            .map(|c| c.get_completed() != 0)
            .unwrap_or_default()
    });

    let countable_type = create_read_slice(selection, move |sel| {
        matches!(
            sel.get(&key.get_untracked())
                .map(|c| c.kind())
                .unwrap_or(CountableKind::Counter(key.get_untracked())),
            CountableKind::Phase(_)
        )
    });

    view! {
        <Overlay show_overlay=show_overlay location=location accent_color=accent_color.unwrap()>
            <ContextMenuNav href=move || format!("edit/{}", key())>
                <span>Edit</span>
            </ContextMenuNav>
            // <ActionForm action=delete_action>
            <Show when=move || countable_type.get()>
                <div
                    class="context-menu-row"
                    on:click=move |ev| {
                        ev.stop_propagation();
                        ev.prevent_default();
                        selection
                            .update(|sel| {
                                if let Some(c) = sel.get(&key.get_untracked()) {
                                    c.toggle_success()
                                }
                            });
                        if let Some(c) = selection.get_untracked().get(&key.get_untracked()) {
                            save_handler.add_countable(c.clone().into());
                            save_handler.save(user.get_untracked());
                        }
                    }
                >

                    {move || if is_success() { "Unmark Success" } else { "Mark Success" }}
                </div>
            </Show>
            // TODO: look further into this actionform not working
            <div
                class="context-menu-row"
                on:click=move |ev| {
                    ev.stop_propagation();
                    ev.prevent_default();
                    delete_action
                        .dispatch(api::RemoveCountable {
                            session: user.get_untracked(),
                            key: key.get_untracked(),
                        });
                    create_effect(move |_| {
                        if !delete_action.pending().get() {
                            state_rsrc.refetch();
                        }
                    });
                }
            >

                Delete
            </div>
        // </ActionForm>
        </Overlay>
    }
}

#[component]
pub fn ContextMenuRow(children: Children) -> impl IntoView {
    view! { <div class="context-menu-row">{children()}</div> }
}

#[component]
pub fn ContextMenuNav<H>(href: H, children: Children) -> impl IntoView
where
    H: ToHref + 'static,
{
    let on_click = move |_| {
        if let Some(t) = use_context::<RwSignal<CloseOverlays>>() {
            t.update(|_| ())
        }
    };

    view! {
        <A href=href class="remove-underline" on:click=on_click>
            <div class="context-menu-row">{children()}</div>
        </A>
    }
}
