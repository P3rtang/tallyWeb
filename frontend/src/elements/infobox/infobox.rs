#![allow(non_snake_case)]
use super::*;

#[derive(Debug, Clone, Copy, Default)]
pub struct IsActive(RwSignal<bool>);
impl IsActive {
    fn toggle(&self) {
        self.0.update(|b| *b = !*b)
    }

    fn set(&self, set: bool) {
        if self.0.get_untracked() != set {
            self.0.update(|b| *b = set);
        }
    }
}

impl FnOnce<()> for IsActive {
    type Output = bool;

    extern "rust-call" fn call_once(self, _: ()) -> Self::Output {
        self.0.get()
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct HasChange(RwSignal<bool>);

impl HasChange {
    fn set(&self, set: bool) {
        self.0.update(|b| *b = set);
    }
}

impl FnOnce<()> for HasChange {
    type Output = bool;

    extern "rust-call" fn call_once(self, _: ()) -> Self::Output {
        self.0.get()
    }
}

#[component]
pub fn InfoBox(#[prop(into)] countable_list: Signal<Vec<CountableId>>) -> impl IntoView {
    // let screen = expect_context::<Screen>();

    // let multi_narrow = move || !(show_multiple() && ScreenStyle::Portrait == (screen.style)());

    view! {
        <div class=style::infobox>
            <For
                each=countable_list
                key=|key| *key
                children=move |key| {
                    view! { <InfoBoxPart key /> }
                }
            />

        </div>
    }
}

#[component]
pub fn InfoBoxPart(#[prop(into)] key: Signal<CountableId>) -> impl IntoView {
    let store = expect_context::<RwSignal<CountableStore>>();
    let show_title = true;
    let on_mobile = use_breakpoint(ViewPort::Medium, true);

    let is_active = IsActive::default();
    provide_context(is_active);
    let has_change = HasChange::default();
    provide_context(has_change);

    let saving = use_saving();

    let descendants = Memo::new(move |_| {
        let store = store.get();
        store
            .recursive_ref()
            .children(&key.get())
            .iter()
            .filter_map(|d| store.get(d))
            .collect::<Vec<_>>()
    });

    Effect::new(move |_| {
        is_active.0.with(|_| {
            has_change.set(false);
            saving(descendants.get_untracked());
        });
    });

    on_cleanup(move || is_active.set(false));

    view! {
        <Show when=move || key.try_get().is_some_and(|key| store.get().contains(&key))>
            <div class=style::row>
                <InfoHeader key />
                <Count key show_title />
                <Time key show_title />
                <Show when=move || !on_mobile.get()>
                    <Progress expand=true key show_title />
                    <LastStep key show_title />
                    <AverageStep key show_title />
                </Show>
            </div>
        </Show>
    }
}

#[component]
fn Count(
    #[prop(into)] key: Signal<CountableId>,
    #[prop(into, optional)] expand: Signal<bool>,
    #[prop(into)] show_title: Signal<bool>,
) -> impl IntoView {
    let store = expect_context::<RwSignal<CountableStore>>();
    let is_active = expect_context::<IsActive>();
    let has_change = expect_context::<HasChange>();

    let get_count = create_read_slice(store, move |s| s.recursive_ref().count(&key.get()));
    let inc_count = create_write_slice(store, move |s, _| s.recursive_ref().increase(&key.get()));
    let add_count = create_write_slice(store, move |s, count| {
        s.recursive_ref().add_count(&key.get(), count)
    });

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
                    inc_count(());
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
        has_change.set(true);
        inc_count(());
    };

    let on_minus_click = move |ev: MouseEvent| {
        has_change.set(true);
        ev.stop_propagation();
        add_count(-1);
    };

    let class = move || {
        stylance::classes! {
            style::rowbox,
            if expand.get() { Some(style::expand) } else { None }
        }
    };

    view! {
        <div class=class on:click=on_count_click data-testid="box">
            <span
                class=style::title
                style:display=move || if show_title() { "block" } else { "none" }
            >
                Count
            </span>
            <span class=style::info data-testid="info">
                {get_count}
            </span>
            <button class=style::count_minus on:click=on_minus_click>
                -
            </button>
        </div>
    }
}

#[cfg(not(feature = "ssr"))] // run timer only on client
struct Handle(IntervalHandle);
// WARN: this is bad but there is no good solution for now
#[cfg(not(feature = "ssr"))] // run timer only on client
unsafe impl Send for Handle {}
#[cfg(not(feature = "ssr"))] // run timer only on client
unsafe impl Sync for Handle {}

#[component]
fn Time(
    #[prop(into)] key: Signal<CountableId>,
    #[prop(into, optional)] expand: Signal<bool>,
    #[prop(into)] show_title: Signal<bool>,
) -> impl IntoView {
    let is_active = expect_context::<IsActive>();
    let has_change = expect_context::<HasChange>();
    let store = expect_context::<RwSignal<CountableStore>>();

    #[allow(unused_variables)]
    let (time, add_time) = create_slice(
        store,
        move |s| {
            s.recursive_ref()
                .time(&key.get())
                .to_std()
                .unwrap_or_default()
        },
        move |s, add| s.recursive_ref().add_time(&key.get(), add),
    );

    #[cfg(not(feature = "ssr"))] // run timer only on client
    {
        let time = signal(0_u32);
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

        if let Ok(handle) = handle {
            let handle = Handle(handle);
            on_cleanup(move || handle.0.clear());
        };
    }

    let class = move || {
        stylance::classes! {
            style::rowbox,
            if expand.get() { Some(style::expand) } else { None }
        }
    };

    let on_click = move |_| {
        has_change.set(true);
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
fn Progress(
    #[prop(into)] key: Signal<CountableId>,
    #[prop(into, optional)] expand: Signal<bool>,
    #[prop(into)] show_title: Signal<bool>,
) -> impl IntoView {
    let store = expect_context::<RwSignal<CountableStore>>();

    let progress = create_read_slice(store, move |s| {
        s.recursive_ref().progress(&key.get_untracked())
    });
    let rolls = create_read_slice(store, move |s| {
        s.recursive_ref().rolls(&key.get_untracked())
    });
    let odds = create_read_slice(store, move |s| s.recursive_ref().odds(&key.get()));

    let color = move || match progress() {
        num if num < 0.5 => "#50fa7b",
        num if num < 0.75 && rolls() < odds() as i32 => "#fcff10",
        num if num < 0.75 => "#ffb86c",
        _ => "#ff9580",
    };

    let classes = move || {
        stylance::classes! {
            style::rowbox,
            if expand.get() { Some(style::expand) } else { None }
        }
    };

    view! {
        <div class=classes>
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
fn LastStep(
    #[prop(into)] key: Signal<CountableId>,
    #[prop(into, optional)] expand: Signal<bool>,
    #[prop(into)] show_title: Signal<bool>,
) -> impl IntoView {
    let store = expect_context::<RwSignal<CountableStore>>();

    let last_interaction = RwSignal::new(None::<i64>);
    let on_count = create_read_slice(store, move |s| s.recursive_ref().count(&key.get()));
    let time = create_read_slice(store, move |s| s.recursive_ref().time(&key.get()));

    let time_value = Memo::new(move |_| {
        on_count.track();
        let val = last_interaction
            .get_untracked()
            .map(|t| time.get_untracked() - Duration::milliseconds(t));
        last_interaction.set(Some(time.get_untracked().num_milliseconds()));
        val
    });

    let format = Memo::new(move |_| {
        time_value.with(|v| {
            match v {
                Some(d) if d.num_hours() > 0 => "%Hh %M",
                Some(d) if d.num_minutes() > 0 => "%Mm %S",
                _ => "%Ss %3f",
            }
            .to_string()
        })
    });

    let classes = move || {
        stylance::classes! {
            style::rowbox,
            expand.get().then_some(style::expand)
        }
    };

    let time_style = || stylance::classes!(style::info, style::time);

    view! {
        <div class=classes>
            <span
                class=style::title
                style:display=move || if show_title() { "block" } else { "none" }
            >
                Last Step
            </span>
            <Show
                when=move || { time_value().is_some() }
                fallback=move || {
                    view! { <span class=time_style>---</span> }
                }
            >
                <components::Timer
                    attr:class=time_style
                    value=time_value().unwrap_or_default().to_std().unwrap_or_default()
                    format
                />
            </Show>
        </div>
    }
}

#[component]
fn AverageStep(
    #[prop(into)] key: Signal<CountableId>,
    #[prop(into, optional)] expand: Signal<bool>,
    #[prop(into)] show_title: Signal<bool>,
) -> impl IntoView {
    let store = expect_context::<RwSignal<CountableStore>>();

    let count = create_read_slice(store, move |s| s.recursive_ref().count(&key.get()));
    let time = create_read_slice(store, move |s| s.recursive_ref().time(&key.get()));

    let step = Memo::new(move |_| {
        Duration::milliseconds(time().num_milliseconds() / count().max(1) as i64)
    });

    let timer_value = Memo::new(move |_| step().to_std().unwrap_or_default());

    let format = Memo::new(move |_| {
        step.with(|v| {
            match v {
                d if d.num_hours() > 0 => "%Hh %M",
                d if d.num_minutes() > 0 => "%Mm %S",
                _ => "%Ss %3f",
            }
            .to_string()
        })
    });

    let class = move || {
        stylance::classes! {
            style::rowbox,
            if expand.get() { Some(style::expand) } else { None }
        }
    };

    let time_style = || stylance::classes!(style::info, style::time);

    view! {
        <div class=class>
            <span
                class=style::title
                style:display=move || if show_title() { "block" } else { "none" }
            >
                Avg Step Time
            </span>
            <Show
                when=move || { count() != 0 }
                fallback=move || {
                    view! { <span class=time_style>---</span> }
                }
            >
                <components::Timer attr:class=time_style value=timer_value format />
            </Show>
        </div>
    }
}
