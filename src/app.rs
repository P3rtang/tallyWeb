#![allow(unused_braces)]
#![allow(non_snake_case)]
use leptos::{*};
use leptos_meta::*;
use leptos_router::*;
use js_sys::Date;

use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use crate::treeview::{*};

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context(cx);

    view! { cx,
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/tally_web.css"/>
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

macro_rules! enclose {
    ( ($( $x:ident ),*) $y:expr ) => {
        {
            $(let $x = $x.clone();)*
            $y
        }
    };
}

#[derive(Debug)]
pub struct ArcCountable(
    Arc<Mutex<Box<dyn Countable>>>,
    Option<RwSignal<i32>>,
    Option<RwSignal<Duration>>,
    Option<Memo<f64>>,
);

impl ArcCountable {
    fn new(countable: Box<dyn Countable>) -> Self {
        return Self(Arc::new(Mutex::new(countable)), None, None, None);
    }

    fn setup_signals(&mut self, cx: Scope) {
        let count_signal = create_rw_signal(cx, self.value());
        self.1 = Some(count_signal);

        let time_signal = create_rw_signal(cx, self.duration());
        self.2 = Some(time_signal);

        let clone = self.clone();
        let progress_signal = create_memo(
            cx,
            enclose!((clone) move |_| {
                count_signal.with(|v| clone.progress(*v))
            }),
        );
        self.3 = Some(progress_signal);

        let _ = self.0.try_lock().map(|mut c| c.get_phases_mut().iter_mut().for_each(|p| p.setup_signals(cx)));
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
        return 1.0 - (1.0 - 1.0_f64 / 8192.0).powi(value)
    }
}

impl IntoNode<ArcCountable> for ArcCountable {
    fn into_node(self, cx: Scope, depth: usize) -> TreeViewNode<ArcCountable> {
        return TreeViewNode::new(
            cx,
            depth,
            self.clone(),
        )
    }

    fn into_view(self, cx: Scope) -> View {
        view! { cx, 
            {self.name()} 
            <button/>
        }.into_view(cx)
    }

    fn get_children(&self) -> Vec<ArcCountable> {
            self.0.try_lock().map_or_else(|_| Vec::new(), |c| c.get_phases().into_iter().map(|p| p.clone()).collect())
    }
}

impl Clone for ArcCountable {
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1, self.2, self.3)
    }
}

impl IntoView for ArcCountable {
    fn into_view(mut self, cx: Scope) -> View {
        if self.1.is_none() || self.2.is_none() || self.3.is_none() {
            self.setup_signals(cx)
        }
        let get_value = self.1.unwrap();
        let get_time = self.2.unwrap();
        let get_progress = self.3.unwrap();

        view! { cx,
        <ul style="display:flex;align-items:center;flex-wrap:wrap">
            <li class="rowbox">
                <p class="title">{ self.name() }</p>
                <p class="info">{ move || get_value() }</p>
            </li>
            <li class="rowbox">
                <p class="title">Time</p>
                <p class="info longtime">{ move || get_time.with(|t| format_time(*t)) }</p>
            </li>
            <li class="rowbox">
                <p class="title">Progress</p>
                <Progressbar progress={
                    move || get_progress()
                } class="info">{
                    move || get_progress.with(|p| format!("{:.03}%", p * 100.0))
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
        }
    };

    create_effect(cx, move |_| {
        set_interval(move || {
            let selection = expect_context::<RwSignal<Selection<ArcCountable>>>(cx);
            selection.get().0
                .into_iter()
                .filter(|(_, b)| *b)
                .for_each(|(c, _)| {
                    let interval = calc_interval(Date::new_0().get_milliseconds(), time.0.get());
                    time.1.set(Date::new_0().get_milliseconds());
                    let _ = c.item().0.try_lock().map(|mut c| c.add_time(interval));
                    c.item().2.map(|s| s.update(|t| *t += interval));
                });
        }, INTERVAL)
    });
}

#[component]
pub fn HomePage(cx: Scope) -> impl IntoView {
    let counter1 = Counter::new("Test 1");
    let counter2 = Counter::new("Test 2");
    let counter3 = Counter::new("Test 3");
    let mut counter4 = Counter::new("Test 4");
    counter4.new_counter("sub counter".to_string());

    let mut list = CounterList::new(&[counter1, counter2, counter3, counter4]);
    list.setup_signals(cx);

    let mut treeview = TreeView::new(cx);
    treeview.set_node(cx, list.list.iter().map(|c| c.clone().into_node(cx, 0)).collect::<Vec<_>>());

    create_effect(
        cx,
        move |_| treeview.selection_signal.with(|sel| sel.0.iter().for_each(|(c, _)| {
            c.item().1.unwrap().set(c.item().value());
            c.item().2.unwrap().set(c.item().duration());
        })),
    );

    let selection = expect_context::<RwSignal<Selection<ArcCountable>>>(cx);
    timer(cx);

    window_event_listener(ev::keypress, {
        move |ev| match ev.code().as_str() {
            "Equal" => {
                selection.with(|list| {
                    list.0.iter().filter(|(_, b)| **b).for_each(|(node, _)| {
                        let _ = node.item().0.try_lock().map(|mut c| {
                            let prev_count = c.get_count();
                            c.set_count(prev_count + 1);
                            node.item().1.map(|s| s.set(prev_count + 1))
                        });
                    })
                })
            }
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
        { treeview.into_view(cx) }
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
                    let rc_counter_signal = create_rw_signal(cx, rc_counter.item().clone());
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

trait Countable: std::fmt::Debug + Send {
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
            .map(|p| {
                p.1.map(|s| s.set(count - diff));
                p.0.try_lock().map(|mut p| p.set_count(count - diff))
            });
    }

    fn add_count(&mut self, count: i32) {
        self.phase_list
            .last_mut()
            .map(|p| {
                p.1.map(|s| s.update(|c| *c += count));
                let _ = p.0.try_lock().map(|mut p| p.add_count(count));
            });
    }

    fn get_time(&self) -> Duration {
        return self.phase_list.iter().map(|p| p.duration()).sum();
    }

    fn set_time(&mut self, time: Duration) {
        self.phase_list
            .last_mut()
            .map(|p| p.0.lock().map(|mut p| p.set_time(time)));
    }

    fn add_time(&mut self, dur: Duration) {
        self.phase_list
            .last_mut()
            .map(|p| {
                p.2.map(|s| s.update(|time| *time += dur));
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

    fn get_phases(&self) -> Vec<&ArcCountable> {
        self.phase_list.iter().collect()
    }

    fn get_phases_mut(&mut self) -> Vec<&mut ArcCountable> {
        return self.phase_list.iter_mut().collect();
    }

    fn new_counter(&mut self, name: String) {
        self.phase_list.push(ArcCountable::new(Box::new(Counter::new(name))))
    }

    fn new_phase(&mut self, name: String) {
        self.phase_list.push(ArcCountable::new(Box::new(Phase::new(name))))
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

    fn get_phases(&self) -> Vec<&ArcCountable> {
        return vec![]
    }

    fn get_phases_mut(&mut self) -> Vec<&mut ArcCountable> {
        return vec![];
    }

    fn new_counter(&mut self, _: String) {
        return ()
    }

    fn new_phase(&mut self, _: String) {
        return ()
    }
}

#[derive(Debug, Clone)]
struct CounterList {
    list: Vec<ArcCountable>,
}

impl CounterList {
    fn new(counters: &[Counter]) -> Self {
        return CounterList {
            list: counters
                .into_iter()
                .map(|c| ArcCountable::new(Box::new(c.clone())))
                .collect(),
        };
    }

    fn setup_signals(&mut self, cx: Scope) {
        self.list.iter_mut().for_each(|c| {
            c.setup_signals(cx);
            c.0.lock().unwrap().get_phases_mut().iter_mut().for_each(|p| {
                p.setup_signals(cx);
            })
        })
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
