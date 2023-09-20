#![allow(unused_braces)]
#![allow(non_snake_case)]

use leptos::{html::Input, *};
use leptos_router::{use_params, Form, IntoParam, Outlet, Params, ParamsError};
use web_sys::SubmitEvent;

use crate::{
    app::{navigate, update_counter, Preferences, SerCounter, SessionUser},
    elements::ScreenLayout,
};

#[component]
pub fn EditWindow(cx: Scope) -> impl IntoView {
    view! { cx,
        <Outlet/>
    }
}

#[component]
pub fn EditCounterWindow<F>(cx: Scope, layout: F) -> impl IntoView
where
    F: Fn() -> ScreenLayout + 'static,
{
    let user = expect_context::<Memo<Option<SessionUser>>>(cx);
    let params = use_params::<CountableId>(cx);

    view! { cx,
    <Show
        when=move || user().is_some()
        fallback=|_| ()>
        { move || {
            let counter_rsrc = create_resource(cx, params, async move |param| {
                let user = user.get_untracked().unwrap_or_default();
                if let Ok(id) = param.map(|p| p.id as i32) {
                    crate::app::get_counter_by_id(user.username, user.token, id).await.map_err(|_| {
                        String::from("Could not access Counter")
                    })
                } else {
                    Err(String::from("Could not get Id"))
                }
            });

            view! { cx, <EditCounterBox counter_rsrc=counter_rsrc/> }
        }}
    </Show>
    }
}

#[derive(Debug, Clone, Params, PartialEq, Eq, Default)]
struct CountableId {
    id: usize,
}

impl std::ops::Deref for CountableId {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.id
    }
}

#[component]
fn EditCounterBox(
    cx: Scope,
    counter_rsrc: Resource<Result<CountableId, ParamsError>, Result<SerCounter, String>>,
) -> impl IntoView {
    let user = expect_context::<Memo<Option<SessionUser>>>(cx);
    let preferences = expect_context::<Memo<Preferences>>(cx);

    let border_style = move || format!("border: 2px solid {}", preferences().accent_color.0);
    let confirm_style = move || format!("background-color: {}", preferences().accent_color.0);
    let undo_changes = move |_| counter_rsrc.refetch();

    let counter = create_rw_signal(cx, None::<SerCounter>);

    let (name, set_name) = create_slice(
        cx,
        counter,
        |c| c.clone().map(|c| c.name).unwrap_or_default(),
        |c, new| {
            c.as_mut().map(|c| c.name = new);
        },
    );

    let name_input: NodeRef<Input> = create_node_ref(cx);
    let on_submit = move |ev: SubmitEvent| {
        ev.prevent_default();
        set_name(name_input().expect("Defined above").value());

        let action = create_action(cx, async move |_: &()| -> Result<(), ServerFnError> {
            let user = user
                .get_untracked()
                .ok_or(ServerFnError::MissingArg(String::from(
                    "User not available",
                )))?;
            let counter =
                counter
                    .get_untracked()
                    .ok_or(ServerFnError::MissingArg(String::from(
                        "Could not find Counter",
                    )))?;

            update_counter(user.username, user.token, counter).await?;
            navigate(cx, "/");
            return Ok(());
        });
        action.dispatch(());
    };

    view! { cx,
        <Transition
            fallback=|| ()
        >
        { move || { counter.set(counter_rsrc.read(cx).map(|c| c.ok()).flatten()); } }
        <Form action="/" on:submit=on_submit class="parent-form">
            <div class="editing-form" style=border_style>
                <EditRow label="name" input=name_input value=name/>
                <div  class="action-buttons">
                    <button type="button" on:click=undo_changes>
                        <span>Undo</span>
                    </button>
                    <button style=confirm_style type="submit">
                        <span>Save</span>
                    </button>
                </div>
            </div>
        </Form>
        </Transition>
    }
}

#[component]
fn EditRow(
    cx: Scope,
    label: &'static str,
    input: NodeRef<Input>,
    value: Signal<String>,
) -> impl IntoView {
    view! { cx,
    <div class="editing-row">
        <label for=label>{ label }</label>
        <input
            type="text"
            name="counter_name"
            node_ref=input
            value=value
        />
    </div>
    }
}
