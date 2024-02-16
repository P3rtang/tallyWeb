use components::MessageBox;
use leptos::*;
use leptos_router::{Outlet, A};

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
    let msg = expect_context::<MessageBox>();
    msg.without_timeout().set_msg("message 1");
    msg.without_timeout()
        .set_msg("message 2 which is a longer message");
    msg.without_timeout()
        .set_msg("message 3\nwith one more line");
    msg.set_msg("message 4\nthis one dissappears");
    msg.without_timeout().set_err("An error occurred")
}
