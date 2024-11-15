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
    let store = expect_context::<RwSignal<CountableStore>>();
    let msg = expect_context::<MessageJar>();

    let delete_action = create_server_action::<api::ArchiveCountable>();
    create_effect(move |_| match delete_action.value()() {
        Some(Ok(_)) => leptos_router::use_navigate()("/", Default::default()),
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
        ev.stop_propagation();
        ev.prevent_default();

        let countable = store.get_untracked().get(&key().into()).unwrap();
        delete_action.dispatch(api::ArchiveCountable { countable });
        store.update(|s| {
            s.recursive_ref().archive(&key.get_untracked().into());
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
            // TODO: look further into this actionform not working
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
