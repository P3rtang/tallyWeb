#![allow(non_snake_case)]

use super::*;
use chrono::Duration;
use components::Progressbar;
use leptos::*;
use web_sys::MouseEvent;

stylance::import_style!(style, "infobox.module.scss");

#[derive(Debug, Clone, Copy, Default)]
pub struct IsActive(RwSignal<bool>);
impl IsActive {
    fn toggle(&self) {
        self.0.update(|b| *b = !*b)
    }

    fn set(&self, set: bool) {
        self.0.update(|b| *b = set);
    }
}

impl FnOnce<()> for IsActive {
    type Output = bool;

    extern "rust-call" fn call_once(self, _: ()) -> Self::Output {
        self.0.get()
    }
}

#[component]
pub fn InfoBox(#[prop(into)] countable_list: Signal<Vec<uuid::Uuid>>) -> impl IntoView {
    let screen = expect_context::<Screen>();
    let store = expect_context::<RwSignal<CountableStore>>();
    let preferences = expect_context::<RwSignal<Preferences>>();

    let show_multiple = move || countable_list().len() > 1;
    let show_title = move || !((screen.style)() == ScreenStyle::Portrait || show_multiple());
    let multi_narrow = move || !(show_multiple() && ScreenStyle::Portrait == (screen.style)());

    view! {
        <div id="infobox" style:display=move || if !multi_narrow() { "block" } else { "flex" }>
            <For
                each=countable_list
                key=|key| *key
                children=move |key| {
                    let is_active = IsActive::default();
                    create_effect(move |_| {
                        let save_handler = expect_context::<RwSignal<SaveHandlers>>();
                        is_active
                            .0
                            .with(|a| {
                                if !a && preferences.get_untracked().save_on_pause {
                                    let _ = save_handler
                                        .get_untracked()
                                        .save(
                                            Box::new(store.get_untracked().last_child(&key.into())),
                                            Box::new(|_| ()),
                                        );
                                }
                            });
                    });
                    provide_context(is_active);
                    view! {
                        // TODO: refactor into a component
                        <Show when=move || store().contains(&key.into())>
                            <div class=style::row>
                                <Show when=show_multiple>
                                    <Title key />
                                </Show>
                                <Count expand=show_multiple key show_title />
                                <Time expand=show_multiple key show_title />
                                <Show when=multi_narrow>
                                    <Progress expand=|| true key show_title />
                                    <LastStep expand=show_multiple key show_title />
                                    <AverageStep expand=show_multiple key show_title />
                                </Show>
                            </div>
                        </Show>
                    }
                }
            />

        </div>
    }
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
fn Title(#[prop(into)] key: MaybeSignal<uuid::Uuid>) -> impl IntoView {
    let state = expect_context::<SelectionSignal>();

    let get_name = create_read_slice(state, move |state| {
        state.get(&key()).map(|c| c.name()).unwrap_or_default()
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
fn Count<T, E>(
    #[prop(into)] key: MaybeSignal<uuid::Uuid>,
    expand: E,
    show_title: T,
) -> impl IntoView
where
    E: Fn() -> bool + Copy + 'static,
    T: Fn() -> bool + Copy + 'static,
{
    let store = expect_context::<RwSignal<CountableStore>>();
    let is_active = expect_context::<IsActive>();
    let name = create_read_slice(store, move |s| s.name(&key().into()));

    let (get_count, add_count) = create_slice(
        store,
        move |s| s.count(&key().into()),
        move |s, count| s.add_count(&key().into(), count),
    );

    let key_listener = window_event_listener(ev::keydown, move |ev| {
        if !document()
            .active_element()
            .map(|e| {
                // TODO: this feels like a hack look into this later
                e.tag_name() == "INPUT"
            })
            .unwrap_or_default()
        {
            match ev.code().as_str() {
                "Equal" => {
                    is_active.set(true);
                    add_count(1);
                }
                "Minus" => {
                    add_count(-1);
                }
                "KeyP" => is_active.toggle(),
                _ => {}
            }
        }
    });

    on_cleanup(|| key_listener.remove());

    let on_count_click = move |_| {
        is_active.set(true);
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
                {name}
            </span>
            <span class=style::info data-testid="info">
                {get_count}
            </span>
        </div>
    }
}

#[component]
fn Time<T, E>(#[prop(into)] key: MaybeSignal<uuid::Uuid>, expand: E, show_title: T) -> impl IntoView
where
    E: Fn() -> bool + Copy + 'static,
    T: Fn() -> bool + Copy + 'static,
{
    let is_active = expect_context::<IsActive>();
    let store = expect_context::<RwSignal<CountableStore>>();

    #[allow(unused_variables)]
    let (time, add_time) = create_slice(
        store,
        move |s| s.time(&key().into()).to_std().unwrap_or_default(),
        move |s, add| s.add_time(&key().into(), add),
    );

    #[cfg(not(feature = "ssr"))] // run timer only on client
    {
        let time = create_signal(0_u32);
        let calc_interval =
            |now: u32, old: u32| Duration::milliseconds(((1000 + now - old) % 1000).into());

        let handle = set_interval_with_handle(
            move || {
                let new_time = js_sys::Date::new_0().get_milliseconds();
                let interval = calc_interval(new_time, time.0.try_get().unwrap_or_default());
                if is_active() {
                    add_time(interval);
                }
                time.1.try_set(new_time);
            },
            std::time::Duration::from_millis(33),
        );

        on_cleanup(|| {
            let _ = handle.map(|h| h.clear());
        });
    }

    let class = move || {
        stylance::classes! {
            style::rowbox,
            if expand() { Some(style::expand) } else { None }
        }
    };

    let on_click = move |_| {
        is_active.toggle();
    };

    view! {
        <div class=class on:click=on_click data-testid="box">
            <span
                class=style::title
                style:display=move || if show_title() { "block" } else { "none" }
            >
                Time
            </span>
            <components::Timer
                attr:class=style::info
                attr:data-testid="info"
                value=time
                format="%H:%M:%S%.3f"
            />
        </div>
    }
}

#[component]
fn Progress<T, E>(
    #[prop(into)] key: MaybeSignal<uuid::Uuid>,
    expand: E,
    show_title: T,
) -> impl IntoView
where
    E: Fn() -> bool + Copy + 'static,
    T: Fn() -> bool + Copy + 'static,
{
    let store = expect_context::<RwSignal<CountableStore>>();

    let progress = create_read_slice(store, move |s| s.progress(&key.get_untracked().into()));
    let rolls = create_read_slice(store, move |s| s.rolls(&key.get_untracked().into()));
    let odds = create_read_slice(store, move |s| s.odds(&key().into()));

    let color = move || match progress() {
        num if num < 0.5 => "#50fa7b",
        num if num < 0.75 && rolls() < odds() as usize => "#fcff10",
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
            <Progressbar progress attr:class=style::info color>

                {move || format!("{:.03}%", progress() * 100.0)}

            </Progressbar>
        </div>
    }
}

#[component]
fn LastStep<E, T>(
    #[prop(into)] key: MaybeSignal<uuid::Uuid>,
    expand: E,
    show_title: T,
) -> impl IntoView
where
    E: Fn() -> bool + Copy + 'static,
    T: Fn() -> bool + Copy + 'static,
{
    let store = expect_context::<RwSignal<CountableStore>>();

    let format_time = |millis: Option<Duration>| match millis {
        None => String::from("---"),
        Some(m) => short_format_time(m),
    };

    let on_count = create_read_slice(store, move |s| s.count(&key().into()));
    let time = create_read_slice(store, move |s| s.time(&key().into()));
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
    #[prop(into)] key: MaybeSignal<uuid::Uuid>,
    expand: E,
    show_title: T,
) -> impl IntoView
where
    E: Fn() -> bool + Copy + 'static,
    T: Fn() -> bool + Copy + 'static,
{
    let store = expect_context::<RwSignal<CountableStore>>();

    let count = create_read_slice(store, move |s| s.count(&key().into()));
    let time = create_read_slice(store, move |s| s.time(&key().into()));

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
