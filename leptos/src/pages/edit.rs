#![allow(unused_braces)]
#![allow(non_snake_case)]

use chrono::Duration;
use components::ScreenLayout;
use leptos::{
    html::{Input, Select},
    *,
};
use leptos_router::{use_params, Form, IntoParam, Outlet, Params, ParamsError};
use web_sys::{Event, SubmitEvent};

use crate::{
    app::{navigate, update_counter, update_phase, Preferences, SessionUser},
    countable::{Countable, Hunttype},
};

#[component]
pub fn EditWindow() -> impl IntoView {
    view! {
        <Outlet/>
    }
}

#[component]
pub fn EditCounterWindow<F>(layout: F) -> impl IntoView
where
    F: Fn() -> ScreenLayout + Copy + 'static,
{
    let user = expect_context::<Memo<Option<SessionUser>>>();
    let params = use_params::<CountableId>();

    view! {
    <Show
        when=move || user().is_some()
        fallback=|| ()
    > { move || {
            let counter_rsrc = create_resource(params, async move |param| {
                let user = user.get_untracked().unwrap_or_default();
                if let Ok(id) = param.map(|p| p.id as i32) {
                    crate::app::get_counter_by_id(user.username, user.token, id).await.map_err(|_| {
                        String::from("Could not access Counter")
                    })
                } else {
                    Err(String::from("Could not get Id"))
                }
            });

            view! { <EditCounterBox layout=layout is_phase=false counter_rsrc=counter_rsrc/> }
        }}
    </Show>
    }
}

#[component]
pub fn EditPhaseWindow<F>(layout: F) -> impl IntoView
where
    F: Fn() -> ScreenLayout + Copy + 'static,
{
    let user = expect_context::<Memo<Option<SessionUser>>>();
    let params = use_params::<CountableId>();

    view! {
    <Show
        when=move || user().is_some()
        fallback=|| ()>
        { move || {
            let phase_rsrc = create_resource(params, async move |param| {
                let user = user.get_untracked().unwrap_or_default();
                if let Ok(id) = param.map(|p| p.id as i32) {
                    crate::app::get_phase_by_id(user.username, user.token, id).await.map_err(|_| {
                        String::from("Could not access Phase")
                    })
                } else {
                    Err(String::from("Could not get Id"))
                }
            });

            view! { <EditCounterBox layout=layout is_phase=true counter_rsrc=phase_rsrc/> }
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
fn EditCounterBox<F, T>(
    layout: F,
    is_phase: bool,
    counter_rsrc: Resource<Result<CountableId, ParamsError>, Result<T, String>>,
) -> impl IntoView
where
    F: Fn() -> ScreenLayout + Copy + 'static,
    T: Countable + Clone,
{
    let user = expect_context::<Memo<Option<SessionUser>>>();
    let preferences = expect_context::<Memo<Preferences>>();

    let message = create_rw_signal(None::<String>);

    let border_style = move || format!("border: 2px solid {}", preferences().accent_color.0);
    let confirm_style = move || format!("background-color: {}", preferences().accent_color.0);

    let counter = create_rw_signal(None::<T>);

    let (name, set_name) = create_slice(
        counter,
        |c| c.clone().map(|c| c.get_name()).unwrap_or_default(),
        |c, new| {
            c.as_mut().map(|c| c.set_name(new));
        },
    );

    let (count, set_count) = create_slice(
        counter,
        |c| c.clone().map(|c| c.get_count()).unwrap_or_default(),
        |c, new| {
            c.as_mut().map(|c| c.set_count(new));
        },
    );

    let (hours, set_hours) = create_slice(
        counter,
        |c| {
            c.clone()
                .map(|c| c.get_time().num_hours())
                .unwrap_or_default()
        },
        |c, new| {
            let old_h = c
                .clone()
                .map(|c| c.get_time().num_hours())
                .unwrap_or_default();
            let diff = Duration::hours(old_h - new);
            c.as_mut().map(|c| c.set_time(c.get_time() - diff));
        },
    );
    let (mins, set_mins) = create_slice(
        counter,
        |c| {
            c.clone()
                .map(|c| c.get_time().num_minutes() % 60)
                .unwrap_or_default()
        },
        |c, new| {
            let old_m = c
                .clone()
                .map(|c| c.get_time().num_minutes() % 60)
                .unwrap_or_default();
            let diff = Duration::minutes(old_m - new);
            c.as_mut().map(|c| c.set_time(c.get_time() - diff));
        },
    );

    let (hunt_type, set_hunt_type) = create_slice(
        counter,
        |c| c.as_ref().map(|c| c.get_hunt_type()).unwrap_or_default(),
        |c, new| {
            let _ = c.as_mut().map(|c| c.set_hunt_type(new));
        },
    );

    let (has_charm, set_charm) = create_slice(
        counter,
        |c| c.as_ref().map(|c| c.has_charm()).unwrap_or_default(),
        |c, new| {
            c.as_mut().map(|c| c.set_charm(new));
        },
    );

    let name_input: NodeRef<Input> = create_node_ref();
    let count_input: NodeRef<Input> = create_node_ref();
    let hours_input: NodeRef<Input> = create_node_ref();
    let mins_input: NodeRef<Input> = create_node_ref();
    let hunt_type_dropdown: NodeRef<Select> = create_node_ref();
    let charm_toggle: NodeRef<Input> = create_node_ref();

    create_effect(move |_| {
        let hunt: String = hunt_type().into();
        hunt_type_dropdown().map(|d| d.set_value(&hunt));
        charm_toggle().map(|t| t.set_checked(has_charm()));
    });

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

        set_hours(
            hours_input()
                .expect("Defined above")
                .value()
                .parse::<i64>()
                .unwrap_or_default(),
        );

        set_mins(
            mins_input()
                .expect("Defined above")
                .value()
                .parse::<i64>()
                .unwrap_or_default(),
        );

        if let Ok(hunt_type) =
            Hunttype::try_from(hunt_type_dropdown().expect("Defined above").value())
        {
            set_hunt_type(hunt_type);
        } else {
            message.set(Some(String::from("Could not save selected Hunttype")));
            return;
        }

        set_charm(charm_toggle().expect("Defined above").checked());

        let action = create_action(async move |_: &()| -> Result<(), ServerFnError> {
            let user = user
                .get_untracked()
                .ok_or(ServerFnError::MissingArg(String::from(
                    "User not available",
                )))
                .map_err(|err| {
                    message.set(Some(String::from("Failed to Save, Could not get user")));
                    err
                })?;
            let counter = counter
                .get_untracked()
                .ok_or(ServerFnError::MissingArg(String::from(
                    "Could not find Counter",
                )))
                .map_err(|err| {
                    message.set(Some(String::from("Failed to Save, Could not get Counter")));
                    err
                })?;

            if is_phase {
                update_phase(
                    user.username,
                    user.token,
                    counter.as_any().downcast_ref().cloned().unwrap(),
                )
                .await
                .map_err(|err| {
                    message.set(Some(format!("Failed to update Phase\nGot error {}", err)));
                    err
                })?;
            } else {
                update_counter(
                    user.username,
                    user.token,
                    counter.as_any().downcast_ref().cloned().unwrap(),
                )
                .await
                .map_err(|err| {
                    message.set(Some(format!("Failed to update Counter\nGot error {}", err)));
                    err
                })?;
            }

            navigate("/");
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

    view! {         <Transition
            fallback=|| ()
        >
            { move || { counter.set(counter_rsrc.get().map(|c| c.ok()).flatten()); } }
            <Form action="/" on:submit=on_submit class="parent-form">
                <div class={ move || String::from("editing-form ") + layout().get_class() } style=border_style>
                    <div class="content">
                        <label for="name" class="title">Name</label>
                        <input type="text" id="name" node_ref=name_input value=name class="edit" autocomplete="none"/>
                        <label for="count" class="title">Count</label>
                        <input type="number" id="count" node_ref=count_input value=count class="edit"/>
                        <label for="time_hours" class="title">Time</label>
                        <span style="display: flex; align-items: center;">
                            <input
                                type="number"
                                id="time_hours"
                                node_ref=hours_input
                                prop:value=hours
                                class="edit"
                                style="width:
                                7ch"
                            />
                            <div style="position: relative; left: -24px;"> H</div>
                            <input
                                type="number"
                                id="time_mins"
                                node_ref=mins_input
                                prop:value=mins
                                on:input=on_mins_input
                                class="edit"
                                style="width: 5ch"
                            />
                            <div style="position: relative; left: -24px;"> M</div>
                        </span>
                        <label for="hunt_type" class="title">Hunting Method</label>
                        <select node_ref=hunt_type_dropdown class="edit" id="hunt_type">
                            <option value="OldOdds">Old odds (1/8192)</option>
                            <option value="NewOdds">New odds (1/4096)</option>
                            <option value="SOS">SOS hunt</option>
                            <option value="MasudaGenIV">Masuda GenIV</option>
                            <option value="MasudaGenV">Masuda GenV</option>
                            <option value="MasudaGenVI">Masuda GenVI+</option>
                        </select>
                        <label for="charm" class="title">Shiny Charm</label>
                        <input type="checkbox" id="charm" node_ref=charm_toggle prop:checked=has_charm.get_untracked() class="edit"/>
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
        <Show
            when=move || { message().is_some() }
            fallback=|| ()
        >
            <b class="notification-box" style=border_style>{ move || { message().unwrap() } }</b>
        </Show>
    }
}
