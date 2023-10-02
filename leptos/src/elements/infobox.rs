use std::time::{Duration, Instant};

use leptos::*;

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
    let state = expect_context::<RwSignal<CounterList>>(cx);
    let on_time_click = move |_| state.update(|s| s.toggle_paused(cx));

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
                let on_count_click = move |_| {
                    countable.update(|c| c.add_count(1));
                    state.update(|s| s.is_paused = false)
                };

                let name = create_read_slice(cx, countable, |c| c.get_name());
                let count = create_read_slice(cx, countable, |c| c.get_count());
                let time = create_read_slice(cx, countable, |c| format_time(c.get_time()));
                let progress = create_read_slice(cx, countable, |c| c.get_progress());

                view! { cx,
                <li class="row">
                    <div class="rowbox" on:click=on_count_click>
                        <p class="title" style=show_title>{ name }</p>
                        <p class="info">{ count }</p>
                    </div>

                    <div class="rowbox" on:click=on_time_click>
                        <p class="title" style=show_title>Time</p>
                        <p class="info longtime">{ time }</p>
                    </div>
                    <div class="rowbox rowexpand">
                        <p class="title" style=show_title>Progress</p>
                        <Progressbar
                            progress=progress
                            class="info"
                        >{ move || format!("{:.03}%", progress() * 100.0) }
                        </Progressbar>
                    </div>
                    <LastStep countable=countable show_title=show_title/>
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

#[component]
fn LastStep<T>(cx: Scope, countable: RwSignal<ArcCountable>, show_title: T) -> impl IntoView
where
    T: Fn() -> String + 'static,
{
    // TODO: just use the duration from the counter itself instead of system time
    let format_time = |millis: Option<u64>| match millis {
        None => return format!("---"),
        Some(m) => format!("{:02}s {:03}", m / 1000, m % 1000),
    };

    let last_interaction = create_rw_signal(cx, None::<u64>);

    let on_count = create_read_slice(cx, countable, |countable| countable.get_count());

    view! { cx,
        <div class="rowbox">
            <p class="title" style=show_title>Last Step</p>
            <p class="info">{ move || {
                on_count.with(|_| {
                    let time = format_time(last_interaction.get_untracked()
                        .map(|time| js_sys::Date::now() as u64 - time));
                    last_interaction.set(Some(js_sys::Date::now() as u64));
                    time
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
    let step = create_read_slice(cx, countable, |c| {
        Duration::from_millis(c.get_time().as_millis() as u64 / c.get_count() as u64)
    });

    view! { cx,
        <div class="rowbox">
            <p class="title" style=show_title>~ Step Time</p>
            <p class="info"> { move || {
                format!("{:02}s {:03}", step().as_secs(), step().as_millis() % 1000)
            }}</p>
        </div>
    }
}
