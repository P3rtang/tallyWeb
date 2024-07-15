#![allow(unused_braces)]
#![allow(non_snake_case)]

use components::Overlay;
use leptos::*;
use leptos_router::A;

use super::*;

#[component]
pub fn CountableContextMenu(
    show_overlay: RwSignal<bool>,
    location: ReadSignal<(i32, i32)>,
    #[prop(into)] key: Signal<uuid::Uuid>,
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

    let is_phase = create_read_slice(selection, move |sel| {
        matches!(
            sel.get(&key.get_untracked())
                .map(|c| c.kind())
                .unwrap_or(CountableKind::Counter),
            CountableKind::Phase
        )
    });

    stylance::import_style!(style, "context_menu.module.scss");
    stylance::import_style!(overlay, "overlay.module.scss");

    view! {
        <Overlay
            attr:class=stylance::classes!(overlay::overlay, style::context_menu)
            show_overlay=show_overlay
            location=location
        >
            <A href=move || format!("edit/{}", key()) class="remove-underline">
                <div class=stylance::classes!(overlay::row, overlay::interactive)>
                    <span>Edit</span>
                </div>
            </A>
            // <ActionForm action=delete_action>
            <Show when=move || is_phase.get()>
                <div
                    class=stylance::classes!(overlay::row, overlay::interactive)
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
                class=stylance::classes!(overlay::row, overlay::interactive)
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
