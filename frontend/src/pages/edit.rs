#![allow(non_snake_case)]
use chrono::Duration;
use components::{LoadingScreen, MessageJar, SavingMessage, Slider};
use leptos::{
    html::{Input, Select},
    *,
};
use leptos_router::{use_params, Form, Outlet, Params};
use web_sys::{Event, SubmitEvent};

use super::*;

#[component]
pub fn EditWindow() -> impl IntoView {
    view! {
        <elements::Navbar></elements::Navbar>
        <Outlet/>
    }
}

#[component]
pub fn EditCounterWindow() -> impl IntoView {
    let params = use_params::<CountableId>();

    let valid_key = params
        .get_untracked()
        .map(|p| uuid::Uuid::parse_str(&p.id).ok())
        .ok()
        .flatten();

    let key = create_rw_signal(
        params
            .get_untracked()
            .map(|p| uuid::Uuid::parse_str(&p.id).ok())
            .ok()
            .flatten()
            .unwrap_or_default(),
    );

    create_isomorphic_effect(move |_| {
        params.with(|p| {
            key.set(
                p.clone()
                    .map(|p| uuid::Uuid::parse_str(&p.id).ok())
                    .ok()
                    .flatten()
                    .unwrap_or_default(),
            )
        });
    });

    view! {
        <Show when=move || valid_key.is_some()>
            <EditCounterBox key/>
        </Show>
    }
}

#[derive(Debug, Clone, Params, PartialEq, Eq, Default)]
struct CountableId {
    id: String,
}

#[component]
fn EditCounterBox(key: RwSignal<uuid::Uuid>) -> impl IntoView {
    let user = expect_context::<RwSignal<UserSession>>();
    let preferences = expect_context::<RwSignal<Preferences>>();
    let message = expect_context::<MessageJar>();
    let session = expect_context::<RwSignal<UserSession>>();

    let counter_rsrc = create_resource(key, move |id| {
        api::get_countable_by_id(session.get_untracked(), id)
    });

    let accent_color = create_read_slice(preferences, |pref| pref.accent_color.0.clone());
    let border_style = move || format!("border: 2px solid {}", accent_color());
    let confirm_style = move || format!("background-color: {}", accent_color());

    let countable = create_rw_signal(None::<ArcCountable>);

    let (name, set_name) = create_slice(
        countable,
        |c| c.clone().map(|c| c.get_name()).unwrap_or_default(),
        |c, new| {
            if let Some(c) = c.as_mut() {
                c.set_name(new)
            }
        },
    );

    let (count, set_count) = create_slice(
        countable,
        |c| c.clone().map(|c| c.get_count()).unwrap_or_default(),
        |c, new| {
            if let Some(c) = c.as_mut() {
                c.set_count(new)
            }
        },
    );

    let (hours, set_hours) = create_slice(
        countable,
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
            if let Some(c) = c.as_mut() {
                c.set_time(c.get_time() - diff)
            }
        },
    );
    let (mins, set_mins) = create_slice(
        countable,
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
            if let Some(c) = c.as_mut() {
                c.set_time(c.get_time() - diff)
            }
        },
    );

    let (hunt_type, set_hunt_type) = create_slice(
        countable,
        |c| c.as_ref().map(|c| c.get_hunt_type()).unwrap_or_default(),
        |c, new| {
            let _ = c.as_mut().map(|c| c.set_hunt_type(new));
        },
    );

    let hunt_type_str = create_read_slice(countable, |c| {
        c.as_ref()
            .map(|c| <&'static str>::from(c.get_hunt_type()).to_string())
            .unwrap_or_default()
    });

    let has_charm = create_slice(
        countable,
        |c| c.as_ref().map(|c| c.has_charm()).unwrap_or_default(),
        |c, new| {
            if let Some(c) = c.as_mut() {
                c.set_charm(new)
            }
        },
    );

    let toggle_charm = move |_| {
        let toggle = !has_charm.0.get_untracked();
        has_charm.1.set(toggle)
    };

    let name_input: NodeRef<Input> = create_node_ref();
    let count_input: NodeRef<Input> = create_node_ref();
    let hours_input: NodeRef<Input> = create_node_ref();
    let mins_input: NodeRef<Input> = create_node_ref();
    let hunt_type_dropdown: NodeRef<Select> = create_node_ref();

    let undo_changes = move |_| {
        counter_rsrc.refetch();
        if let Some(v) = name_input() {
            v.set_value(&name())
        }
        if let Some(v) = count_input() {
            v.set_value_as_number(count() as f64)
        }
        if let Some(v) = hours_input() {
            v.set_value_as_number(hours() as f64)
        }
        if let Some(v) = mins_input() {
            v.set_value_as_number(mins() as f64)
        }
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
            message.set_err("Could not save\nselected Hunttype");
            return;
        }

        let action = create_action(move |(user, countable): &(UserSession, ArcCountable)| {
            api::update_countable(user.clone(), countable.clone().try_into().unwrap())
        });

        action.dispatch((user(), countable().unwrap()));

        create_effect(move |_| {
            match action.value()() {
                Some(Ok(_)) => {
                    message.set_msg("Saved succesfully");
                    leptos_router::use_navigate()("/", Default::default());
                }
                Some(Err(err)) => message.set_err(err.to_string()),
                _ => {}
            };
        });

        message
            .without_timeout()
            .as_modal()
            .set_msg_view(SavingMessage)
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

    view! {
        <Transition fallback=move || {
            view! { <LoadingScreen/> }
        }>
            {move || {
                countable.set(counter_rsrc.get().and_then(|c| c.ok()).map(|c| c.into()));
            }}
            <Form action="/" on:submit=on_submit class="parent-form">
                <div class=move || String::from("editing-form ") style=border_style>
                    <div class="content">
                        <label for="name" class="title">
                            Name
                        </label>
                        <input
                            type="text"
                            id="name"
                            node_ref=name_input
                            value=name
                            class="edit"
                            autocomplete="none"
                        />
                        <label for="count" class="title">
                            Count
                        </label>
                        <input
                            type="number"
                            id="count"
                            node_ref=count_input
                            value=count
                            class="edit"
                        />
                        <label for="time_hours" class="title">
                            Time
                        </label>
                        <span style="display: flex; align-items: center;">
                            <input
                                type="number"
                                id="time_hours"
                                node_ref=hours_input
                                value=hours
                                class="edit"
                                style="width:
                                7ch"
                            />
                            <div style="position: relative; left: -24px;">H</div>
                            <input
                                type="number"
                                id="time_mins"
                                node_ref=mins_input
                                value=mins
                                on:input=on_mins_input
                                class="edit"
                                style="width: 5ch"
                            />
                            <div style="position: relative; left: -24px;">M</div>
                        </span>
                        <label for="hunt_type" class="title">
                            Hunting Method
                        </label>
                        <select
                            node_ref=hunt_type_dropdown
                            class="edit"
                            id="hunt_type"
                            value=hunt_type_str
                        >

                            {
                                create_isomorphic_effect(move |_| {
                                    let hunt: &'static str = hunt_type().into();
                                    if let Some(d) = hunt_type_dropdown() {
                                        d.set_value(&hunt)
                                    }
                                });
                            }

                            <option value="NewOdds">New odds (1/4096)</option>
                            <option value="OldOdds">Old odds (1/8192)</option>
                            <option value="SOS">SOS hunt</option>
                            <option value="MasudaGenIV">Masuda GenIV</option>
                            <option value="MasudaGenV">Masuda GenV</option>
                            <option value="MasudaGenVI">Masuda GenVI+</option>
                        </select>
                        <label for="charm" class="title">
                            Shiny Charm
                        </label>
                        <Slider checked=has_charm.0 on:change=toggle_charm/>
                    </div>
                    <div class="action-buttons">
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
