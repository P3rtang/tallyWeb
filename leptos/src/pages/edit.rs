#![allow(unused_braces)]
#![allow(non_snake_case)]

use std::time::Duration;

use leptos::{html::Input, *};
use leptos_router::{use_params, Form, IntoParam, Outlet, Params, ParamsError};
use web_sys::{Event, SubmitEvent};

use crate::{
    app::{
        navigate, update_counter, update_phase, Countable, Phase, Preferences, SerCounter,
        SessionUser,
    },
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
    F: Fn() -> ScreenLayout + Copy + 'static,
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

            view! { cx, <EditCounterBox layout=layout counter_rsrc=counter_rsrc/> }
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
fn EditCounterBox<F>(
    cx: Scope,
    layout: F,
    counter_rsrc: Resource<Result<CountableId, ParamsError>, Result<SerCounter, String>>,
) -> impl IntoView
where
    F: Fn() -> ScreenLayout + Copy + 'static,
{
    let user = expect_context::<Memo<Option<SessionUser>>>(cx);
    let preferences = expect_context::<Memo<Preferences>>(cx);

    let border_style = move || format!("border: 2px solid {}", preferences().accent_color.0);
    let confirm_style = move || format!("background-color: {}", preferences().accent_color.0);

    let counter = create_rw_signal(cx, None::<SerCounter>);

    let (name, set_name) = create_slice(
        cx,
        counter,
        |c| c.clone().map(|c| c.name).unwrap_or_default(),
        |c, new| {
            c.as_mut().map(|c| c.name = new);
        },
    );

    let (count, set_count) = create_slice(
        cx,
        counter,
        |c| c.clone().map(|c| c.get_count()).unwrap_or_default(),
        |c, new| {
            c.as_mut().map(|c| c.set_count(new));
        },
    );

    let (hours, set_hours) = create_slice(
        cx,
        counter,
        |c| {
            c.clone()
                .map(|c| c.get_time().as_secs() / 3600)
                .unwrap_or_default()
        },
        |c, new| {
            let old_h = c
                .clone()
                .map(|c| c.get_time().as_secs() / 3600)
                .unwrap_or_default();
            if new < old_h as i32 {
                let diff = Duration::from_secs((old_h - new as u64) * 3600);
                c.as_mut().map(|c| c.rem_time(diff));
            } else {
                let diff = Duration::from_secs((new as u64 - old_h) * 3600);
                c.as_mut().map(|c| c.add_time(diff));
            }
        },
    );
    let (mins, set_mins) = create_slice(
        cx,
        counter,
        |c| {
            c.clone()
                .map(|c| c.get_time().as_secs() / 60 % 60)
                .unwrap_or_default()
        },
        |c, new| {
            let old_m = c
                .clone()
                .map(|c| c.get_time().as_secs() / 60 % 60)
                .unwrap_or_default();
            if new < old_m as i32 {
                let diff = Duration::from_secs((old_m - new as u64) * 60);
                c.as_mut().map(|c| c.rem_time(diff));
            } else {
                let diff = Duration::from_secs((new as u64 - old_m) * 60);
                c.as_mut().map(|c| c.add_time(diff));
            }
        },
    );

    let name_input: NodeRef<Input> = create_node_ref(cx);
    let count_input: NodeRef<Input> = create_node_ref(cx);
    let hours_input: NodeRef<Input> = create_node_ref(cx);
    let mins_input: NodeRef<Input> = create_node_ref(cx);

    let undo_changes = move |_| {
        counter_rsrc.refetch();
        name_input().map(|v| v.set_value(&name()));
        count_input().map(|v| v.set_value_as_number(count() as f64));
        hours_input().map(|v| v.set_value_as_number(hours() as f64));
        mins_input().map(|v| v.set_value_as_number(mins() as f64));
    };

    let on_submit = move |ev: SubmitEvent| {
        ev.prevent_default();
        set_name(name_input().expect("Defined above").value());
        if let Ok(num) = count_input().expect("Defined above").value().parse::<i32>() {
            set_count(num)
        }

        if let Ok(num) = hours_input().expect("Defined above").value().parse::<i32>() {
            set_hours(num);
        }

        if let Ok(num) = mins_input().expect("Defined above").value().parse::<i32>() {
            set_mins(num);
        }

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

    let on_mins_input = move |ev: Event| {
        if let Ok(num) = event_target_value(&ev).parse::<i32>() {
            if num > 60 {
                mins_input().unwrap().set_value("59")
            } else if num <= 0 {
                mins_input().unwrap().set_value("00")
            }
        }
    };

    view! { cx,
        <Transition
            fallback=|| ()
        >
            { move || { counter.set(counter_rsrc.read(cx).map(|c| c.ok()).flatten()); } }
            <Form action="/" on:submit=on_submit class="parent-form">
                <div class={ move || String::from("editing-form ") + layout().get_class() } style=border_style>
                    <div class="content">
                        <label for="name" class="title">Name</label>
                        <input type="text" name="name" node_ref=name_input value=name class="edit"/>
                        <label for="count" class="title">Count</label>
                        <input type="number" name="count" node_ref=count_input value=count class="edit"/>
                        <label for="time" class="title">Time</label>
                        <span style="display: flex; align-items: center;">
                            <input
                                type="number"
                                name="time"
                                node_ref=hours_input
                                prop:value=hours
                                class="edit"
                                style="width:
                                7ch"
                            />
                            <div style="position: relative; left: -24px;"> H</div>
                            <input
                                type="number"
                                name="time"
                                node_ref=mins_input
                                prop:value=mins
                                on:input=on_mins_input
                                class="edit"
                                style="width: 5ch"
                            />
                            <div style="position: relative; left: -24px;"> M</div>
                        </span>
                    </div>
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
pub fn EditPhaseWindow<F>(cx: Scope, layout: F) -> impl IntoView
where
    F: Fn() -> ScreenLayout + Copy + 'static,
{
    let user = expect_context::<Memo<Option<SessionUser>>>(cx);
    let params = use_params::<CountableId>(cx);

    view! { cx,
    <Show
        when=move || user().is_some()
        fallback=|_| ()>
        { move || {
            let phase_rsrc = create_resource(cx, params, async move |param| {
                let user = user.get_untracked().unwrap_or_default();
                if let Ok(id) = param.map(|p| p.id as i32) {
                    crate::app::get_phase_by_id(user.username, user.token, id).await.map_err(|_| {
                        String::from("Could not access Counter")
                    })
                } else {
                    Err(String::from("Could not get Id"))
                }
            });

            view! { cx, <EditPhaseBox layout=layout phase_rsrc=phase_rsrc/> }
        }}
    </Show>
    }
}

#[component]
fn EditPhaseBox<F>(
    cx: Scope,
    layout: F,
    phase_rsrc: Resource<Result<CountableId, ParamsError>, Result<Phase, String>>,
) -> impl IntoView
where
    F: Fn() -> ScreenLayout + Copy + 'static,
{
    let user = expect_context::<Memo<Option<SessionUser>>>(cx);
    let preferences = expect_context::<Memo<Preferences>>(cx);

    let border_style = move || format!("border: 2px solid {}", preferences().accent_color.0);
    let confirm_style = move || format!("background-color: {}", preferences().accent_color.0);

    let phase = create_rw_signal(cx, None::<Phase>);

    let (name, set_name) = create_slice(
        cx,
        phase,
        |c| c.clone().map(|c| c.name).unwrap_or_default(),
        |c, new| {
            c.as_mut().map(|c| c.name = new);
        },
    );

    let (count, set_count) = create_slice(
        cx,
        phase,
        |c| c.clone().map(|c| c.get_count()).unwrap_or_default(),
        |c, new| {
            c.as_mut().map(|c| c.set_count(new));
        },
    );

    let (hours, set_hours) = create_slice(
        cx,
        phase,
        |c| {
            c.clone()
                .map(|c| c.get_time().as_secs() / 3600)
                .unwrap_or_default()
        },
        |c, new| {
            let old_h = c
                .clone()
                .map(|c| c.get_time().as_secs() / 3600)
                .unwrap_or_default();
            if new < old_h as i32 {
                let diff = Duration::from_secs((old_h - new as u64) * 3600);
                c.as_mut().map(|c| c.rem_time(diff));
            } else {
                let diff = Duration::from_secs((new as u64 - old_h) * 3600);
                c.as_mut().map(|c| c.add_time(diff));
            }
        },
    );
    let (mins, set_mins) = create_slice(
        cx,
        phase,
        |c| {
            c.clone()
                .map(|c| c.get_time().as_secs() / 60 % 60)
                .unwrap_or_default()
        },
        |c, new| {
            let old_m = c
                .clone()
                .map(|c| c.get_time().as_secs() / 60 % 60)
                .unwrap_or_default();
            if new < old_m as i32 {
                let diff = Duration::from_secs((old_m - new as u64) * 60);
                c.as_mut().map(|c| c.rem_time(diff));
            } else {
                let diff = Duration::from_secs((new as u64 - old_m) * 60);
                c.as_mut().map(|c| c.add_time(diff));
            }
        },
    );

    let name_input: NodeRef<Input> = create_node_ref(cx);
    let count_input: NodeRef<Input> = create_node_ref(cx);
    let hours_input: NodeRef<Input> = create_node_ref(cx);
    let mins_input: NodeRef<Input> = create_node_ref(cx);

    let undo_changes = move |_| {
        phase_rsrc.refetch();
        name_input().map(|v| v.set_value(&name()));
        count_input().map(|v| v.set_value_as_number(count() as f64));
        hours_input().map(|v| v.set_value_as_number(hours() as f64));
        mins_input().map(|v| v.set_value_as_number(mins() as f64));
    };

    let on_submit = move |ev: SubmitEvent| {
        ev.prevent_default();
        set_name(name_input().expect("Defined above").value());
        if let Ok(num) = count_input().expect("Defined above").value().parse::<i32>() {
            set_count(num)
        }

        if let Ok(num) = hours_input().expect("Defined above").value().parse::<i32>() {
            set_hours(num);
        }

        if let Ok(num) = mins_input().expect("Defined above").value().parse::<i32>() {
            set_mins(num);
        }

        let action = create_action(cx, async move |_: &()| -> Result<(), ServerFnError> {
            let user = user
                .get_untracked()
                .ok_or(ServerFnError::MissingArg(String::from(
                    "User not available",
                )))?;
            let counter = phase
                .get_untracked()
                .ok_or(ServerFnError::MissingArg(String::from(
                    "Could not find Counter",
                )))?;

            update_phase(user.username, user.token, counter).await?;
            navigate(cx, "/");
            return Ok(());
        });
        action.dispatch(());
    };

    let on_mins_input = move |ev: Event| {
        if let Ok(num) = event_target_value(&ev).parse::<i32>() {
            if num > 60 {
                mins_input().unwrap().set_value("59")
            } else if num <= 0 {
                mins_input().unwrap().set_value("00")
            }
        }
    };

    view! { cx,
        <Transition
            fallback=|| ()
        >
            { move || { phase.set(phase_rsrc.read(cx).map(|c| c.ok()).flatten()); } }
            <Form action="/" on:submit=on_submit class="parent-form">
                <div class={ move || String::from("editing-form ") + layout().get_class() } style=border_style>
                    <div class="content">
                        <label for="name" class="title">Name</label>
                        <input type="text" name="name" node_ref=name_input value=name class="edit"/>
                        <label for="count" class="title">Count</label>
                        <input type="number" name="count" node_ref=count_input value=count class="edit"/>
                        <label for="time" class="title">Time</label>
                        <span style="display: flex; align-items: center;">
                            <input
                                type="number"
                                name="time"
                                node_ref=hours_input
                                prop:value=hours
                                class="edit"
                                style="width:
                                7ch"
                            />
                            <div style="position: relative; left: -24px;"> H</div>
                            <input
                                type="number"
                                name="time"
                                node_ref=mins_input
                                prop:value=mins
                                on:input=on_mins_input
                                class="edit"
                                style="width: 5ch"
                            />
                            <div style="position: relative; left: -24px;"> M</div>
                        </span>
                    </div>
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
