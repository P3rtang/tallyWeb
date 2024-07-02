use super::*;
use components::MessageJar;
use leptos::*;
use leptos_router::{Outlet, A};

#[server]
async fn failing_server_fn() -> Result<(), ServerFnError> {
    use super::AppError;
    return Err(AppError::Internal)?;
}

#[component]
pub fn ShowTests() -> impl IntoView {
    let test_list = vec![("TestMessages", "messages")]
        .into_iter()
        .map(|(key, href)| {
            view! {
                <div class="test-entry">
                    <A href>
                        <span>{key}</span>
                    </A>
                </div>
            }
        })
        .collect_view();

    view! { <test-list>{test_list} <Outlet/></test-list> }
}

#[component]
pub fn TestMessages() -> impl IntoView {
    let failed_action = create_server_action::<FailingServerFn>();
    failed_action.dispatch(FailingServerFn {});
    let msg = expect_context::<MessageJar>();

    let server_resp = create_memo(move |_| {
        if let Some(Err(err)) = failed_action.value().get() {
            msg.without_timeout().set_err(AppError::from(err))
        }
    });

    create_effect(move |_| server_resp.track());

    msg.without_timeout().set_msg("message 1");
    msg.without_timeout()
        .set_msg("message 2 which is a longer message");
    msg.without_timeout()
        .set_msg("message 3\nwith one more line");
    msg.with_timeout(chrono::Duration::seconds(3))
        .set_msg("message 4\nthis one dissappears");
    msg.without_timeout().set_err("An error occurred")
}
