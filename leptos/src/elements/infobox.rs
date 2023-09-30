use leptos::*;

use crate::{
    app::{ArcCountable, CounterList, Progressbar},
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
                    <div class="rowbox">
                        <p class="title" style=show_title>Progress</p>
                        <Progressbar
                            progress=progress
                            class="info"
                        >{ move || format!("{:.03}%", progress() * 100.0) }
                        </Progressbar>
                    </div>
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
