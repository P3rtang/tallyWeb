#![allow(unused_braces)]
#![allow(non_snake_case)]

use components::{MessageJar, Overlay};
use leptos::*;
use leptos_router::A;

use super::*;

stylance::import_style!(style, "context_menu.module.scss");
stylance::import_style!(overlay, "overlay.module.scss");

#[component]
pub fn CountableContextMenu(
    show_overlay: RwSignal<bool>,
    location: ReadSignal<(i32, i32)>,
    #[prop(into)] key: MaybeSignal<uuid::Uuid>,
) -> impl IntoView {
    let user = expect_context::<RwSignal<UserSession>>();
    let store = expect_context::<RwSignal<CountableStore>>();
    let state_rsrc = expect_context::<StateResource>();
    let msg = expect_context::<MessageJar>();
    // TODO: reintroduce saving
    // let save_handler = expect_context::<SaveHandlerCountable>();
    //
    let delete_action = create_server_action::<api::RemoveCountable>();
    create_effect(move |_| match delete_action.value()() {
        Some(Ok(_)) => state_rsrc.refetch(),
        Some(Err(err)) => msg.set_server_err(&err),
        None => {}
    });

    let is_phase = create_read_slice(store, move |s| {
        matches!(
            s.get(&key.get_untracked().into()),
            Some(Countable::Phase(_))
        )
    });

    let (is_success, toggle_success) = create_slice(
        store,
        move |s| s.is_success(&key().into()),
        move |s, _| s.toggle_success(&key().into()),
    );

    let on_click_delete = move |ev: ev::MouseEvent| {
        let countable = store.get_untracked().get(&key().into()).unwrap();
        ev.stop_propagation();
        ev.prevent_default();
        delete_action.dispatch(api::RemoveCountable {
            session: user.get_untracked(),
            countable,
        });
    };

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
                        toggle_success(());
                    }
                >

                    {move || if is_success() { "Unmark Success" } else { "Mark Success" }}
                </div>
            </Show>
            // TODO: look further into this actionform not working
            <div
                class=stylance::classes!(overlay::row, overlay::interactive)
                on:click=on_click_delete
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
