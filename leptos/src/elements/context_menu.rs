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
    let delete_action = create_server_action::<api::RemoveCountable>();

    view! {
        <Overlay show_overlay=show_overlay location=location accent_color=accent_color.unwrap()>
            <ContextMenuNav href=move || format!("edit/{}", key().to_string())>
                <span>Edit</span>
            </ContextMenuNav>
            // TODO: look further into this actionform not working
            // <ActionForm action=delete_action>
            <div
                class="context-menu-row"
                type="submit"
                on:click=move |ev| {
                    ev.stop_propagation();
                    ev.prevent_default();
                    delete_action
                        .dispatch(api::RemoveCountable {
                            session: user.get_untracked(),
                            key: key.get_untracked(),
                        });
                    state_rsrc.refetch();
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
