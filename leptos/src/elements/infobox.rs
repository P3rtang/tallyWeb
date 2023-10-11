#![allow(non_snake_case)]

use chrono::Duration;
use leptos::*;

use crate::{
    app::{CounterList, Progressbar},
    countable::ArcCountable,
    elements::ScreenLayout,
};

#[component]
pub fn InfoBox(countable_list: RwSignal<Vec<RwSignal<ArcCountable>>>) -> impl IntoView {
    let screen_layout = expect_context::<RwSignal<ScreenLayout>>();
    let show_title = move || {
        if screen_layout() == ScreenLayout::Small {
            format!("display: none")
        } else {
            String::new()
        }
    };

    view! {
        <ul id="InfoBox">
        <For
            each=countable_list
            key=|countable| countable().get_uuid()
            children=move |countable| {
                view! {
                <li class="row">
                    <Count countable show_title/>
                    <Time countable show_title/>
                    <Progress countable show_title/>
                    <LastStep countable show_title/>
                    <AverageStep countable show_title/>
                </li>
                }
            }
        >
        </For>
        </ul>
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
    return match dur {
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
    };
}

#[component]
fn Count<T>(countable: RwSignal<ArcCountable>, show_title: T) -> impl IntoView
where
    T: Fn() -> String + 'static,
{
    let state = expect_context::<RwSignal<CounterList>>();

    let on_count_click = move |_| {
        countable.update(|c| c.add_count(1));
        state.update(|s| s.start())
    };

    let name = create_read_slice(countable, |c| c.get_name());
    let count = create_read_slice(countable, |c| c.get_count());

    view! {
        <div class="rowbox" on:click=on_count_click>
            <p class="title" style=show_title>{ name }</p>
            <p class="info">{ count }</p>
        </div>
    }
}

#[component]
fn Time<T>(countable: RwSignal<ArcCountable>, show_title: T) -> impl IntoView
where
    T: Fn() -> String + 'static,
{
    let state = expect_context::<RwSignal<CounterList>>();
    let on_time_click = move |_| state.update(|s| s.toggle_paused());

    let time = create_read_slice(countable, |c| format_time(c.get_time()));

    view! {
        <div class="rowbox" on:click=on_time_click>
            <p class="title" style=show_title>Time</p>
            <p class="info longtime">{ time }</p>
        </div>
    }
}

#[component]
fn Progress<T>(countable: RwSignal<ArcCountable>, show_title: T) -> impl IntoView
where
    T: Fn() -> String + 'static,
{
    let progress = create_read_slice(countable, |c| c.get_progress());

    let color = create_read_slice(countable, |c| match c.get_progress() {
        num if num < 0.5 => "#50fa7b",
        num if num < 0.75 && c.get_rolls() < c.get_odds() => "#fcff10",
        num if num < 0.75 => "#ffb86c",
        _ => "#ff9580",
    });

    view! {
        <div class="rowbox rowexpand">
            <p class="title" style=show_title>Progress</p>
            <Progressbar
                progress=progress
                class="info"
                color
            >
                { move || format!("{:.03}%", progress() * 100.0) }
            </Progressbar>
        </div>
    }
}

#[component]
fn LastStep<T>(countable: RwSignal<ArcCountable>, show_title: T) -> impl IntoView
where
    T: Fn() -> String + 'static,
{
    // TODO: just use the duration from the counter itself instead of system time
    let format_time = |millis: Option<Duration>| match millis {
        None => return format!("---"),
        Some(m) => short_format_time(m),
    };

    let on_count = create_read_slice(countable, |c| c.get_count());
    let time = create_read_slice(countable, |c| c.get_time());
    let last_interaction = create_rw_signal(None::<i64>);

    view! {
        <div class="rowbox">
            <p class="title" style=show_title>Last Step</p>
            <p class="info">{ move || {
                on_count.with(|_| {
                    let time_str = format_time(last_interaction.get_untracked().map(|t| {
                        time.get_untracked() - Duration::milliseconds(t)
                    }));
                    last_interaction.set(Some(time.get_untracked().num_milliseconds()));
                    time_str
                })}
            }</p>
        </div>
    }
}

#[component]
fn AverageStep<T>(countable: RwSignal<ArcCountable>, show_title: T) -> impl IntoView
where
    T: Fn() -> String + 'static,
{
    let count = create_read_slice(countable, |c| c.get_count());
    let step = create_read_slice(countable, move |c| {
        Duration::milliseconds(c.get_time().num_milliseconds() / count() as i64)
    });

    view! {
        <div class="rowbox">
            <p class="title" style=show_title>Avg Step Time</p>
            <p class="info"> { move || {
                if count() == 0 {
                    String::from("---")
                } else {
                    short_format_time(step())
                }
            }}</p>
        </div>
    }
}
