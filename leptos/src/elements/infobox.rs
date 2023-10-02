#![allow(non_snake_case)]

use leptos::*;
use std::time::Duration;

use crate::{
    app::{CounterList, Progressbar},
    countable::ArcCountable,
    elements::ScreenLayout,
};

#[component]
pub fn InfoBox<F>(cx: Scope, countable_list: F) -> impl IntoView
where
    F: Fn() -> Vec<RwSignal<ArcCountable>> + 'static,
{
    let screen_layout = expect_context::<RwSignal<ScreenLayout>>(cx);
    let show_title = move || {
        if screen_layout() == ScreenLayout::Small {
            format!("display: none")
        } else {
            String::new()
        }
    };

    view! { cx,
        <ul id="InfoBox">
        <For
            each=countable_list
            key=|countable| countable().get_uuid()
            view=move |cx, countable| {
                view! { cx,
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

fn format_time(dur: std::time::Duration) -> String {
    format!(
        "{:>02}:{:02}:{:02},{:03}",
        dur.as_secs() / 3600,
        dur.as_secs() / 60 % 60,
        dur.as_secs() % 60,
        dur.as_millis() - dur.as_secs() as u128 * 1000,
    )
}

fn short_format_time(dur: std::time::Duration) -> String {
    const HOUR: u64 = 60 * 60 * 1000;
    const MINUTE: u64 = 60 * 1000;
    const SECOND: u64 = 1000;

    return match dur.as_millis() as u64 {
        millis if millis > HOUR => format!("{:02}h {:02}m", millis / HOUR, millis % HOUR / MINUTE),
        millis if millis > MINUTE => {
            format!("{:02}m {:02}s", millis / MINUTE, millis % MINUTE / SECOND)
        }
        millis => {
            format!("{:02}s {:03}", millis / SECOND, millis % SECOND)
        }
    };
}

#[component]
fn Count<T>(cx: Scope, countable: RwSignal<ArcCountable>, show_title: T) -> impl IntoView
where
    T: Fn() -> String + 'static,
{
    let state = expect_context::<RwSignal<CounterList>>(cx);

    let on_count_click = move |_| {
        countable.update(|c| c.add_count(1));
        state.update(|s| s.is_paused = false)
    };

    let name = create_read_slice(cx, countable, |c| c.get_name());
    let count = create_read_slice(cx, countable, |c| c.get_count());

    view! {cx,
        <div class="rowbox" on:click=on_count_click>
            <p class="title" style=show_title>{ name }</p>
            <p class="info">{ count }</p>
        </div>
    }
}

#[component]
fn Time<T>(cx: Scope, countable: RwSignal<ArcCountable>, show_title: T) -> impl IntoView
where
    T: Fn() -> String + 'static,
{
    let state = expect_context::<RwSignal<CounterList>>(cx);
    let on_time_click = move |_| state.update(|s| s.toggle_paused(cx));

    let time = create_read_slice(cx, countable, |c| format_time(c.get_time()));

    view! { cx,
        <div class="rowbox" on:click=on_time_click>
            <p class="title" style=show_title>Time</p>
            <p class="info longtime">{ time }</p>
        </div>
    }
}

#[component]
fn Progress<T>(cx: Scope, countable: RwSignal<ArcCountable>, show_title: T) -> impl IntoView
where
    T: Fn() -> String + 'static,
{
    let progress = create_read_slice(cx, countable, |c| c.get_progress());

    view! { cx,
        <div class="rowbox rowexpand">
            <p class="title" style=show_title>Progress</p>
            <Progressbar
                progress=progress
                class="info"
            >
                { move || format!("{:.03}%", progress() * 100.0) }
            </Progressbar>
        </div>
    }
}

#[component]
fn LastStep<T>(cx: Scope, countable: RwSignal<ArcCountable>, show_title: T) -> impl IntoView
where
    T: Fn() -> String + 'static,
{
    // TODO: just use the duration from the counter itself instead of system time
    let format_time = |millis: Option<Duration>| match millis {
        None => return format!("---"),
        Some(m) => short_format_time(m),
    };

    let on_count = create_read_slice(cx, countable, |c| c.get_count());
    let time = create_read_slice(cx, countable, |c| c.get_time());
    let last_interaction = create_rw_signal(cx, None::<u128>);

    view! { cx,
        <div class="rowbox">
            <p class="title" style=show_title>Last Step</p>
            <p class="info">{ move || {
                on_count.with(|_| {
                    let time_str = format_time(last_interaction.get_untracked().map(|t| {
                        time.get_untracked() - Duration::from_millis(t as u64)
                    }));
                    last_interaction.set(Some(time.get_untracked().as_millis()));
                    time_str
                })}
            }</p>
        </div>
    }
}

#[component]
fn AverageStep<T>(cx: Scope, countable: RwSignal<ArcCountable>, show_title: T) -> impl IntoView
where
    T: Fn() -> String + 'static,
{
    let count = create_read_slice(cx, countable, |c| c.get_count());
    let step = create_read_slice(cx, countable, move |c| {
        Duration::from_millis(c.get_time().as_millis() as u64 / count() as u64)
    });

    view! { cx,
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
