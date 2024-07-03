#![allow(non_snake_case)]

use super::*;
use chrono::Duration;
use components::SidebarStyle;
use leptos::*;
use web_sys::MouseEvent;

use crate::{
    app::{Progressbar, SelectionSignal},
    countable::ArcCountable,
};

stylance::import_style!(style, "infobox.module.scss");

#[component]
pub fn InfoBox(countable_list: Signal<Vec<ArcCountable>>) -> impl IntoView {
    let screen_layout = expect_context::<RwSignal<SidebarStyle>>();
    let show_multiple = move || countable_list().len() > 1;
    let show_title = move || !(screen_layout() == SidebarStyle::Portrait || show_multiple());
    let multi_narrow = move || !(show_multiple() && SidebarStyle::Portrait == screen_layout());

    view! {
        <div id="infobox" style:display=move || if !multi_narrow() { "block" } else { "flex" }>
            <For
                each=countable_list
                key=|countable| countable.get_uuid()
                children=move |countable| {
                    let key = create_signal(countable.get_uuid()).0;
                    view! {
                        <div class=style::row>
                            <Show when=show_multiple>
                                <Title key/>
                            </Show>
                            <Count expand=show_multiple key show_title/>
                            <Time expand=show_multiple key show_title/>
                            <Show when=multi_narrow>
                                <Progress expand=|| true key show_title/>
                                <LastStep expand=show_multiple key show_title/>
                                <AverageStep expand=show_multiple key show_title/>
                            </Show>
                        </div>
                    }
                }
            />

        </div>
    }
}

fn format_time(dur: Duration) -> String {
    format!(
        "{:>02}:{:02}:{:02},{:03}",
        dur.num_hours(),
        dur.num_minutes() % 60,
        dur.num_seconds() % 60,
        dur.num_milliseconds() - dur.num_seconds() * 1000,
    )
}

fn short_format_time(dur: Duration) -> String {
    match dur {
        dur if dur.num_hours() > 0 => {
            format!("{:02}h {:02}m", dur.num_hours(), dur.num_minutes() % 60)
        }
        dur if dur.num_minutes() > 0 => {
            format!("{:02}m {:02}s", dur.num_minutes(), dur.num_seconds() % 60)
        }
        dur => {
            format!(
                "{:02}s {:03}",
                dur.num_seconds(),
                dur.num_milliseconds() % 1000
            )
        }
    }
}

#[component]
fn Title(#[prop(into)] key: Signal<uuid::Uuid>) -> impl IntoView {
    let state = expect_context::<SelectionSignal>();

    let get_name = create_read_slice(state, move |state| {
        state.get(&key()).map(|c| c.get_name()).unwrap_or_default()
    });

    view! {
        <div class="rowbox rowexpand">
            <span
                class=style::info
                style:min-height="0em"
                style:padding="0.5em"
                style:font-size="28px"
            >
                {get_name}
            </span>
        </div>
    }
}

#[component]
fn Count<T, E>(#[prop(into)] key: Signal<uuid::Uuid>, expand: E, show_title: T) -> impl IntoView
where
    E: Fn() -> bool + Copy + 'static,
    T: Fn() -> bool + Copy + 'static,
{
    let user = expect_context::<RwSignal<UserSession>>();
    let state = expect_context::<SelectionSignal>();

    let get_name = create_read_slice(state, move |state| {
        state.get(&key()).map(|c| c.get_name()).unwrap_or_default()
    });

    let toggle_paused = create_write_slice(state, move |s, _| {
        if let Some(item) = s.get_mut(&key()) {
            item.set_active(!item.is_active());
            let save_handler = expect_context::<SaveHandlerCountable>();
            save_handler.add_countable(item.clone().into());
            save_handler.save(user.get_untracked())
        }
    });

    let unpause = create_write_slice(state, move |s, _| {
        if let Some(item) = s.get_mut(&key()) {
            item.set_active(true)
        };
    });

    let (get_count, add_count) = create_slice(
        state,
        move |state| state.get(&key()).map(|c| c.get_count()).unwrap_or_default(),
        move |state, count| {
            if let Some(c) = state.get(&key()) {
                c.add_count(count);
                let save_handler = expect_context::<SaveHandlerCountable>();
                save_handler.add_countable(c.clone().into());
            }
        },
    );

    let key_listener = window_event_listener(ev::keypress, move |ev| match ev.code().as_str() {
        "Equal" => {
            unpause(());
            add_count(1);
        }
        "Minus" => {
            add_count(-1);
        }
        "KeyP" => toggle_paused(()),
        _ => {}
    });

    on_cleanup(|| key_listener.remove());

    let on_count_click = move |_| {
        unpause(());
        add_count(1);
    };

    let on_minus_click = move |ev: MouseEvent| {
        ev.stop_propagation();
        add_count(-1);
    };

    let class = move || {
        stylance::classes! {
            style::rowbox,
            if expand() { Some(style::expand) } else { None }
        }
    };

    view! {
        <div class=class on:click=on_count_click data-testid="box">
            <button class=style::count_minus on:click=on_minus_click>
                -
            </button>
            <span
                class=style::title
                style:display=move || if show_title() { "block" } else { "none" }
            >
                {get_name}
            </span>
            <span class=style::info data-testid="info">
                {get_count}
            </span>
        </div>
    }
}

#[component]
fn Time<T, E>(#[prop(into)] key: Signal<uuid::Uuid>, expand: E, show_title: T) -> impl IntoView
where
    E: Fn() -> bool + Copy + 'static,
    T: Fn() -> bool + Copy + 'static,
{
    let state = expect_context::<SelectionSignal>();
    let user = expect_context::<RwSignal<UserSession>>();

    let toggle_paused = create_write_slice(state, move |s, _| {
        if let Some(item) = s.get_mut(&key()) {
            item.set_active(!item.is_active());
            let save_handler = expect_context::<SaveHandlerCountable>();
            save_handler.add_countable(item.clone().into());
            save_handler.save(user.get_untracked())
        }
    });

    let time = create_read_slice(state, move |state| {
        format_time(
            state
                .get(&key())
                .map(|c| c.get_time())
                .unwrap_or(Duration::zero()),
        )
    });

    let class = move || {
        stylance::classes! {
            style::rowbox,
            if expand() { Some(style::expand) } else { None }
        }
    };

    view! {
        <div class=class on:click=toggle_paused data-testid="box">
            <span
                class=style::title
                style:display=move || if show_title() { "block" } else { "none" }
            >
                Time
            </span>
            <span class=style::info style:min-width="7em" data-testid="info">
                {time}
            </span>
        </div>
    }
}

#[component]
fn Progress<T, E>(#[prop(into)] key: Signal<uuid::Uuid>, expand: E, show_title: T) -> impl IntoView
where
    E: Fn() -> bool + Copy + 'static,
    T: Fn() -> bool + Copy + 'static,
{
    let state = expect_context::<SelectionSignal>();

    let progress = create_read_slice(state, move |state| {
        state
            .get(&key())
            .map(|c| c.get_progress())
            .unwrap_or_default()
    });

    let rolls = create_read_slice(state, move |state| {
        state.get(&key()).map(|c| c.get_rolls()).unwrap_or_default()
    });

    let odds = create_read_slice(state, move |state| {
        state.get(&key()).map(|c| c.get_odds()).unwrap_or_default()
    });

    let color = move || match progress() {
        num if num < 0.5 => "#50fa7b",
        num if num < 0.75 && rolls() < odds() => "#fcff10",
        num if num < 0.75 => "#ffb86c",
        _ => "#ff9580",
    };

    let class = move || {
        stylance::classes! {
            style::rowbox,
            if expand() { Some(style::expand) } else { None }
        }
    };

    view! {
        <div class=class>
            <span
                class=style::title
                style:display=move || if show_title() { "block" } else { "none" }
            >
                Progress
            </span>
            <Progressbar progress class=style::info color>
                {move || format!("{:.03}%", progress() * 100.0)}
            </Progressbar>
        </div>
    }
}

#[component]
fn LastStep<E, T>(#[prop(into)] key: Signal<uuid::Uuid>, expand: E, show_title: T) -> impl IntoView
where
    E: Fn() -> bool + Copy + 'static,
    T: Fn() -> bool + Copy + 'static,
{
    let state = expect_context::<SelectionSignal>();

    let format_time = |millis: Option<Duration>| match millis {
        None => String::from("---"),
        Some(m) => short_format_time(m),
    };

    let on_count = create_read_slice(state, move |state| {
        state.get(&key()).map(|c| c.get_count()).unwrap_or_default()
    });

    let time = create_read_slice(state, move |state| {
        state
            .get(&key())
            .map(|c| c.get_time())
            .unwrap_or(Duration::zero())
    });

    let last_interaction = create_rw_signal(None::<i64>);

    let class = move || {
        stylance::classes! {
            style::rowbox,
            if expand() { Some(style::expand) } else { None }
        }
    };

    view! {
        <div class=class>
            <span
                class=style::title
                style:display=move || if show_title() { "block" } else { "none" }
            >
                Last Step
            </span>
            <span class=stylance::classes!(
                style::info, style::time
            )>
                {move || {
                    on_count
                        .with(|_| {
                            let time_str = format_time(
                                last_interaction
                                    .get_untracked()
                                    .map(|t| { time.get_untracked() - Duration::milliseconds(t) }),
                            );
                            last_interaction.set(Some(time.get_untracked().num_milliseconds()));
                            time_str
                        })
                }}

            </span>
        </div>
    }
}

#[component]
fn AverageStep<E, T>(
    #[prop(into)] key: Signal<uuid::Uuid>,
    expand: E,
    show_title: T,
) -> impl IntoView
where
    E: Fn() -> bool + Copy + 'static,
    T: Fn() -> bool + Copy + 'static,
{
    let state = expect_context::<SelectionSignal>();

    let count = create_read_slice(state, move |state| {
        state.get(&key()).map(|c| c.get_count()).unwrap_or_default()
    });

    let time = create_read_slice(state, move |state| {
        state
            .get(&key())
            .map(|c| c.get_time())
            .unwrap_or(Duration::zero())
    });

    let step = move || Duration::milliseconds(time().num_milliseconds() / count() as i64);

    let class = move || {
        stylance::classes! {
            style::rowbox,
            if expand() { Some(style::expand) } else { None }
        }
    };

    view! {
        <div class=class>
            <span
                class=style::title
                style:display=move || if show_title() { "block" } else { "none" }
            >
                Avg Step Time
            </span>
            <span class=stylance::classes!(
                style::info, style::time
            )>
                {move || {
                    if count() == 0 { String::from("---") } else { short_format_time(step()) }
                }}

            </span>
        </div>
    }
}
