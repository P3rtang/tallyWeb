use super::*;
use components::{self, MessageJar, ShowSidebar, Sidebar, SidebarLayout};
use elements::{FromClosure, OnResize, Page, PageContent, PageSidebar};
use leptos::*;
use leptos_router::{Outlet, Route, A};

stylance::import_style!(style, "tests.module.scss");

#[server]
async fn failing_server_fn() -> Result<(), ServerFnError> {
    use super::AppError;
    return Err(AppError::Internal)?;
}

#[component(transparent)]
pub fn TestRoutes() -> impl IntoView {
    view! {
        <Route path="/test" view=ShowTests>
            <Route path="" view=|| () />
            <Route path="message" view=Message />
            <Route path="slider" view=Slider />
        </Route>
    }
}

#[component]
pub fn ShowTests() -> impl IntoView {
    let test_list = StoredValue::new(
        vec![("Messages", "message"), ("Slider", "slider")]
            .into_iter()
            .map(|(key, href)| {
                view! {
                    <A href>
                        <div class=style::entry>
                            <span>{key}</span>
                        </div>
                    </A>
                }
            })
            .collect_view(),
    );

    let (width, set_width) = create_signal(400);
    let on_resize = OnResize::from_closure(set_width);

    view! {
        <Page>
            <PageContent slot>
                <Outlet />
            </PageContent>
            <PageSidebar is_shown=true auto_hide=true on_resize slot>
                <Sidebar layout=SidebarLayout::Landscape width>
                    <test-list>{test_list()}</test-list>
                </Sidebar>
            </PageSidebar>
        </Page>
    }
}

#[component]
fn Message() -> impl IntoView {
    expect_context::<RwSignal<ShowSidebar>>();
    let msg = expect_context::<MessageJar>();

    let failed_action = create_server_action::<FailingServerFn>();
    failed_action.dispatch(FailingServerFn {});

    create_effect(move |_| {
        if let Some(Err(err)) = failed_action.value().get() {
            msg.without_timeout().set_err(AppError::from(err))
        }
    });

    create_effect(move |_| {
        msg.without_timeout().set_msg("message 1");
        msg.without_timeout()
            .set_msg("message 2 which is a longer message");
        msg.without_timeout()
            .set_msg("message 3\nwith one more line");
        msg.with_timeout(chrono::Duration::seconds(3))
            .set_msg("message 4\nthis one dissappears");
        msg.without_timeout().set_err("An error occurred")
    });
}

#[component]
fn Slider() -> impl IntoView {
    expect_context::<RwSignal<ShowSidebar>>().set(ShowSidebar(false));
    let checked_signal = create_rw_signal(false);
    let disable_signal = create_rw_signal(false);
    let background = create_rw_signal(false);

    view! {
        <div>
            <div class=style::test_case>
                <components::Slider attr:id="test-background" checked=true></components::Slider>
            </div>
            <div class=style::test_case>
                <button data-testid="toggle" on:click=move |_| checked_signal.update(|s| *s = !*s)>
                    Toggle
                </button>
                <components::Slider
                    attr:id="test-managed"
                    checked=checked_signal
                ></components::Slider>
            </div>
            <div class=style::test_case>
                <input
                    type="checkbox"
                    id="check-disable"
                    data-testid="check-disable"
                    prop:checked=move || !disable_signal()
                    checked=move || !disable_signal()
                    on:change=move |_| disable_signal.update(|s| *s = !*s)
                />
                <components::Slider
                    attr:id="test-disable"
                    attr:data-testid="disable"
                    checked=true
                    attr:disabled=disable_signal
                ></components::Slider>
            </div>

            <div class=style::test_case>
                <components::Slider
                    attr:id="test-on_checked"
                    attr:data-testid="on_checked"
                    checked=false
                    on:change=move |_: ev::Event| background.update(|s| *s = !*s)
                ></components::Slider>
                <div
                    attr:data-testid="colored_div"
                    style:width="32px"
                    style:height="32px"
                    style:background=move || if !background() { "#FF0000" } else { "#00FF00" }
                ></div>

            </div>
        </div>
    }
}
