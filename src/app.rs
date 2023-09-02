#![allow(unused_braces)]
#![allow(non_snake_case)]
use js_sys::Date;
use leptos::{ev::MouseEvent, *};
use leptos_meta::*;
use leptos_router::*;

use std::{
    cmp::PartialEq,
    sync::{Arc, Mutex},
    time::Duration,
};

use crate::treeview::*;

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context(cx);

    view! { cx,
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet href="/pkg/tally_web.css"/>
        <Stylesheet href="/stylers.css"/>
        <Script src="https://kit.fontawesome.com/7173474e94.js" crossorigin="anonymous"/>
        <Link rel="shortcut icon" type_="image/ico" href="/favicon.svg"/>

        // sets the document title
        <Title text="TallyWeb"/>

        // content for this welcome page
        <Router>
            <main>
                <Routes>
                    <Route path="" view=HomePage/>
                    <Route path="/*any" view=NotFound/>
                </Routes>
            </main>
        </Router>
    }
}

/// 404 - Not Found
#[component]
fn NotFound(cx: Scope) -> impl IntoView {
    // set an HTTP status code 404
    // this is feature gated because it can only be done during
    // initial server-side rendering
    // if you navigate to the 404 page subsequently, the status
    // code will not be set because there is not a new HTTP request
    // to the server
    #[cfg(feature = "ssr")]
    {
        // this can be done inline because it's synchronous
        // if it were async, we'd use a server function
        let resp = expect_context::<leptos_actix::ResponseOptions>(cx);
        resp.set_status(actix_web::http::StatusCode::NOT_FOUND);
    }

    view! { cx,
        <h1>"Not Found"</h1>
    }
}

#[derive(Debug, Clone)]
pub struct ArcCountable(Arc<Mutex<Box<dyn Countable>>>);

impl ArcCountable {
    fn new(countable: Box<dyn Countable>) -> Self {
        return Self(Arc::new(Mutex::new(countable)));
    }

    fn name(&self) -> String {
        self.0.try_lock().map(|c| c.get_name()).unwrap_or_default()
    }

    fn value(&self) -> i32 {
        self.0.try_lock().map(|c| c.get_count()).unwrap_or_default()
    }

    fn duration(&self) -> Duration {
        self.0.try_lock().map(|c| c.get_time()).unwrap_or_default()
    }

    fn progress(&self, value: i32) -> f64 {
        return 1.0 - (1.0 - 1.0_f64 / 8192.0).powi(value);
    }

    fn new_phase(&self) -> ArcCountable {
        let n_phase = self
            .0
            .try_lock()
            .map(|c| c.get_phases().len() + 1)
            .unwrap_or(1);
        let _ = self
            .0
            .try_lock()
            .map(|mut c| c.new_phase(format!("Phase {n_phase}")));
        self.get_children().last().cloned().unwrap()
    }
}

impl PartialEq for ArcCountable {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(&*self.0, &*other.0)
    }
}

impl Eq for ArcCountable {}

impl core::hash::Hash for ArcCountable {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        std::ptr::hash(&*self.0, state)
    }
}

impl std::ops::Deref for ArcCountable {
    type Target = Mutex<Box<dyn Countable>>;

    fn deref(&self) -> &Self::Target {
        return &*self.0;
    }
}

impl IntoNode<ArcCountable> for ArcCountable {
    fn into_node(self, cx: Scope, depth: usize) -> TreeViewNode<ArcCountable> {
        return TreeViewNode::new(cx, self.clone(), depth);
    }

    fn into_view(self, cx: Scope) -> View {
        let node_children = expect_context::<SignalNodeChildren<ArcCountable>>(cx);
        let selection = expect_context::<RwSignal<Selection<ArcCountable>>>(cx);

        let item = create_rw_signal(cx, self.clone());
        let get_node = create_read_slice(cx, node_children, move |node_signal| {
            node_signal.get(&item.get()).cloned().unwrap()
        });

        let (sel_slice, _) = create_slice(
            cx,
            selection,
            move |sel| sel.0.get(&item()).cloned().unwrap_or_default(),
            move |sel, n| {
                sel.0.insert(item(), n);
            },
        );

        let click_new_phase = move |e: MouseEvent| {
            e.stop_propagation();
            item().new_phase();
            get_node().children.set(
                item()
                    .get_children()
                    .into_iter()
                    .map(|c| c.into_node(cx, get_node().depth + 1))
                    .collect(),
            )
        };

        let div_class = move || {
            let mut class = String::from("selectable row");
            if sel_slice.get() {
                class += " selected"
            }

            return class;
        };

        view! { cx,
            <div>
                <TreeViewRow item=get_node().clone() class=div_class>
                <div class="row-body">
                    <span> { self.name() } </span>
                    <Show when= move || {
                        item.get().0.try_lock().map(|c| c.has_children()).unwrap_or_default()
                    }
                    fallback= move |_| {}
                    ><button on:click=click_new_phase>+</button>
                    </Show>
                </div>
                </TreeViewRow>
            </div>
        }
        .into_view(cx)
    }

    fn get_children(&self) -> Vec<ArcCountable> {
        self.0
            .try_lock()
            .map_or_else(
                |_| Vec::new(),
                |c| c.get_phases().into_iter().map(|p| p.clone()).collect(),
            )
            .clone()
    }
}

impl IntoView for ArcCountable {
    fn into_view(self, cx: Scope) -> View {
        let state = expect_context::<RwSignal<CounterList>>(cx);
        let item_s = create_rw_signal(cx, self.clone());

        let (get_count, _) = create_slice(
            cx,
            state,
            move |_| {
                item_s
                    .get()
                    .0
                    .try_lock()
                    .map(|c| c.get_count())
                    .unwrap_or_default()
            },
            move |_, count| {
                let _ = item_s.get().0.try_lock().map(|mut c| c.set_count(count));
            },
        );

        let (get_time, _) = create_slice(
            cx,
            state,
            move |_| {
                item_s
                    .get()
                    .0
                    .try_lock()
                    .map(|c| c.get_time())
                    .unwrap_or_default()
            },
            move |_, time| {
                let _ = item_s.get().0.try_lock().map(|mut c| c.set_time(time));
            },
        );

        view! { cx,
        <ul style="display:flex;align-items:center;flex-wrap:wrap">
            <li class="rowbox">
                <p class="title">{ self.name() }</p>
                <p class="info">{ move || get_count() }</p>
            </li>
            <li class="rowbox">
                <p class="title">Time</p>
                <p class="info longtime">{ move || get_time.with(|t| format_time(*t)) }</p>
            </li>
            <li class="rowbox">
                <p class="title">Progress</p>
                <Progressbar progress={
                    move || item_s.get().progress(get_count())
                } class="info">{
                    move || get_count.with(|p| format!("{:.03}%", item_s.get().progress(*p) * 100.0))
                }</Progressbar>
            </li>
        </ul>
        }
        .into_view(cx)
    }
}

fn format_time(dur: Duration) -> String {
    format!(
        "{:>02}:{:02}:{:02},{:03}",
        dur.as_secs() / 3600,
        dur.as_secs() / 60 % 60,
        dur.as_secs() % 60,
        dur.as_millis() - dur.as_secs() as u128 * 1000,
    )
}

fn timer(cx: Scope) {
    const FRAMERATE: u64 = 600;
    const INTERVAL: Duration = Duration::from_millis(1000 / FRAMERATE);

    let time = create_signal(cx, 0_u32);

    let calc_interval = |now: u32, old: u32| -> Duration {
        return if now < old {
            Duration::from_millis((1000 + now - old).into())
        } else {
            Duration::from_millis((now - old).into())
        };
    };

    create_effect(cx, move |_| {
        set_interval(
            move || {
                let state = expect_context::<RwSignal<CounterList>>(cx);
                let selection = expect_context::<RwSignal<Selection<ArcCountable>>>(cx);
                if state.get().is_paused {
                    return;
                }
                selection
                    .get()
                    .0
                    .into_iter()
                    .filter(|(_, b)| *b)
                    .for_each(|(c, _)| {
                        let item_s = create_rw_signal(cx, c.clone());

                        let (get_time, set_time) = create_slice(
                            cx,
                            state,
                            move |_| {
                                item_s
                                    .get()
                                    .0
                                    .try_lock()
                                    .map(|c| c.get_time())
                                    .unwrap_or_default()
                            },
                            move |_, time| {
                                let _ = item_s.get().0.try_lock().map(|mut c| c.set_time(time));
                            },
                        );

                        let interval =
                            calc_interval(Date::new_0().get_milliseconds(), time.0.get());
                        time.1.set(Date::new_0().get_milliseconds());
                        set_time.set(get_time() + interval);
                    });
            },
            INTERVAL,
        )
    });
}

#[component]
pub fn HomePage(cx: Scope) -> impl IntoView {
    let counter1 = Counter::new("Test 1");
    let counter2 = Counter::new("Test 2");
    let counter3 = Counter::new("Test 3");
    let mut counter4 = Counter::new("Test 4");
    counter4.new_counter("sub counter".to_string());

    let list = CounterList::new(&[counter1, counter2, counter3, counter4]);
    let state = create_rw_signal(cx, list);
    provide_context(cx, state);
    let treeview = TreeView::new(cx, state.get_untracked().list);

    let selection = expect_context::<RwSignal<Selection<ArcCountable>>>(cx);
    timer(cx);

    window_event_listener(ev::keypress, {
        move |ev| match ev.code().as_str() {
            "Equal" => selection.with(|list| {
                list.0.iter().filter(|(_, b)| **b).for_each(|(node, _)| {
                    let state = expect_context::<RwSignal<CounterList>>(cx);
                    let item_s = create_rw_signal(cx, node.0.clone());

                    let (get_count, set_count) = create_slice(
                        cx,
                        state,
                        move |_| {
                            item_s
                                .get()
                                .try_lock()
                                .map(|c| c.get_count())
                                .unwrap_or_default()
                        },
                        move |_, count| {
                            let _ = item_s.get().try_lock().map(|mut c| c.set_count(count));
                        },
                    );

                    set_count(get_count() + 1);
                })
            }),
            "KeyP" => state.update(|list| list.is_paused = !list.is_paused),
            _ => (),
        }
    });

    view! { cx,
        <div id="HomeGrid">
            <CounterTreeView treeview=treeview/>
            <InfoBox/>
        </div>
    }
}

#[component]
fn CounterTreeView(cx: Scope, treeview: TreeView<ArcCountable>) -> impl IntoView {
    view! { cx,
        { treeview }
    }
}

#[component]
fn Progressbar<F>(cx: Scope, progress: F, class: &'static str, children: Children) -> impl IntoView
where
    F: Fn() -> f64 + 'static,
{
    view! { cx,
        <div style="display: grid; justify-items: center" class=class>
            <div style="font-size: 0.6em; color: gray">{ children(cx) }</div>
            <progress
                max=1
                value=progress
            />
        </div>
    }
}

#[component]
fn InfoBox(cx: Scope) -> impl IntoView {
    let selection = expect_context::<RwSignal<Selection<ArcCountable>>>(cx);
    view! { cx,
        <div id="InfoBox"> {
            move || selection.with(|list| {
                list.0.iter().filter(|(_, b)| **b).map(|(rc_counter, _)| {
                    let rc_counter_signal = create_rw_signal(cx, rc_counter.clone());
                    view! {cx, <InfoBoxRow counter=rc_counter_signal/> }
                }).collect_view(cx)
            })
        } </div>
    }
}

#[component]
fn InfoBoxRow(cx: Scope, counter: RwSignal<ArcCountable>) -> impl IntoView {
    view! { cx,
        <div class="row">
            <div> { counter } </div>
        </div>
    }
}

pub trait Countable: std::fmt::Debug + Send {
    fn get_name(&self) -> String;
    fn get_count(&self) -> i32;
    fn set_count(&mut self, count: i32);
    fn add_count(&mut self, count: i32);
    fn get_time(&self) -> Duration;
    fn set_time(&mut self, dur: Duration);
    fn add_time(&mut self, dur: Duration);
    fn get_progress(&self) -> f64;
    fn is_active(&self) -> bool;
    fn toggle_active(&mut self);
    fn set_active(&mut self, active: bool);

    fn new_phase(&mut self, name: String);
    fn new_counter(&mut self, name: String);

    fn get_phases(&self) -> Vec<&ArcCountable>;
    fn get_phases_mut(&mut self) -> Vec<&mut ArcCountable>;

    fn has_children(&self) -> bool;
}

#[derive(Debug, Clone)]
struct Counter {
    name: String,
    phase_list: Vec<ArcCountable>,

    is_active: bool,
}

#[allow(dead_code)]
impl Counter {
    fn new(name: impl ToString) -> Counter {
        let phase = ArcCountable::new(Box::new(Phase::new("Phase 1")));
        return Counter {
            name: name.to_string(),
            phase_list: vec![phase],
            is_active: false,
        };
    }
}

impl Countable for Counter {
    fn get_name(&self) -> String {
        return self.name.clone();
    }

    fn get_count(&self) -> i32 {
        return self.phase_list.iter().map(|p| p.value()).sum();
    }

    fn set_count(&mut self, count: i32) {
        let diff = self.phase_list.iter().map(|p| p.value()).sum::<i32>()
            - self.phase_list.last().map(|p| p.value()).unwrap_or(0);
        self.phase_list
            .last_mut()
            .map(|p| p.0.try_lock().map(|mut p| p.set_count(count - diff)));
    }

    fn add_count(&mut self, count: i32) {
        self.phase_list.last_mut().map(|p| {
            let _ = p.0.try_lock().map(|mut p| p.add_count(count));
        });
    }

    fn get_time(&self) -> Duration {
        return self.phase_list.iter().map(|p| p.duration()).sum();
    }

    fn set_time(&mut self, time: Duration) {
        let diff = self
            .phase_list
            .iter()
            .map(|p| p.duration())
            .sum::<Duration>()
            - self
                .phase_list
                .last()
                .map(|p| p.duration())
                .unwrap_or_default();
        self.phase_list
            .last_mut()
            .map(|p| p.0.lock().map(|mut p| p.set_time(time - diff)));
    }

    fn add_time(&mut self, dur: Duration) {
        self.phase_list.last_mut().map(|p| {
            let _ = p.0.try_lock().map(|mut p| p.add_time(dur));
        });
    }

    fn get_progress(&self) -> f64 {
        return 1.0 - (1.0 - 1.0_f64 / 8192.0).powi(self.get_count());
    }

    fn is_active(&self) -> bool {
        return self.is_active;
    }

    fn toggle_active(&mut self) {
        self.is_active = !self.is_active
    }

    fn set_active(&mut self, active: bool) {
        self.is_active = active;
        if !active {
            self.phase_list.iter().for_each(|p| {
                let _ = p.0.lock().map(|mut p| p.set_active(false));
            })
        };
    }

    fn new_phase(&mut self, name: String) {
        self.phase_list
            .push(ArcCountable::new(Box::new(Phase::new(name))))
    }

    fn new_counter(&mut self, name: String) {
        self.phase_list
            .push(ArcCountable::new(Box::new(Counter::new(name))))
    }

    fn get_phases(&self) -> Vec<&ArcCountable> {
        self.phase_list.iter().collect()
    }

    fn get_phases_mut(&mut self) -> Vec<&mut ArcCountable> {
        return self.phase_list.iter_mut().collect();
    }

    fn has_children(&self) -> bool {
        true
    }
}

#[derive(Debug, Clone)]
struct Phase {
    name: String,
    count: i32,
    time: Duration,
    is_active: bool,
}

impl Phase {
    fn new(name: impl ToString) -> Self {
        return Phase {
            name: name.to_string(),
            count: 0,
            time: Duration::ZERO,
            is_active: false,
        };
    }
}

impl Countable for Phase {
    fn get_name(&self) -> String {
        return self.name.clone();
    }

    fn get_count(&self) -> i32 {
        return self.count;
    }

    fn set_count(&mut self, count: i32) {
        self.count = count
    }

    fn add_count(&mut self, count: i32) {
        self.count += count
    }

    fn get_time(&self) -> Duration {
        return self.time;
    }

    fn set_time(&mut self, dur: Duration) {
        self.time = dur
    }

    fn add_time(&mut self, dur: Duration) {
        self.time += dur
    }

    fn get_progress(&self) -> f64 {
        return 1.0 - (1.0 - 1.0_f64 / 8192.0).powi(self.get_count());
    }

    fn is_active(&self) -> bool {
        return self.is_active;
    }

    fn toggle_active(&mut self) {
        self.is_active = !self.is_active
    }

    fn set_active(&mut self, active: bool) {
        self.is_active = active
    }

    fn new_phase(&mut self, _: String) {
        return ();
    }

    fn new_counter(&mut self, _: String) {
        return ();
    }

    fn get_phases(&self) -> Vec<&ArcCountable> {
        return vec![];
    }

    fn get_phases_mut(&mut self) -> Vec<&mut ArcCountable> {
        return vec![];
    }

    fn has_children(&self) -> bool {
        false
    }
}

#[derive(Debug, Clone)]
struct CounterList {
    list: Vec<ArcCountable>,
    is_paused: bool,
}

impl CounterList {
    fn new(counters: &[Counter]) -> Self {
        return CounterList {
            list: counters
                .into_iter()
                .map(|c| ArcCountable::new(Box::new(c.clone())))
                .collect(),
            is_paused: true,
        };
    }
}

impl IntoView for CounterList {
    fn into_view(self, cx: Scope) -> View {
        let view = view! { cx,
            <ul>
            </ul>
        };
        return view.into_view(cx);
    }
}

impl std::ops::Index<usize> for CounterList {
    type Output = ArcCountable;

    fn index(&self, index: usize) -> &Self::Output {
        &self.list[index]
    }
}
