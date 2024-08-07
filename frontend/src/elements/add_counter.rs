use super::*;
use components::{MessageJar, Spinner};
use leptos::*;
use stylance::import_style;

#[component]
pub fn NewCounterButton(state_len: Signal<usize>) -> impl IntoView {
    let user = expect_context::<RwSignal<UserSession>>();
    let message = expect_context::<MessageJar>();

    let add_counter_action = create_action(move |(user, name): &(UserSession, String)| {
        let mut new_counter = Counter::new(name, user.user_uuid);
        new_counter.new_phase("Phase 1".to_string());
        api::update_counter(user.clone(), new_counter.into())
    });

    create_effect(move |_| {
        match add_counter_action.value()() {
            Some(Ok(_)) => {
                expect_context::<StateResource>().refetch();
                message.clear();
            }
            Some(Err(err)) => message.set_err(err.to_string()),
            None => {}
        };
    });

    let on_click = move |_| {
        add_counter_action.dispatch((user.get_untracked(), format!("Counter {}", state_len() + 1)));

        message.set_msg_view(view! {
            <div style="display: flex; align-items: center;">
                <Spinner/>
                <b style="font-size: 20px; padding-left: 24px;">Creating Counter</b>
            </div>
        })
    };

    import_style!(style, "new-counter.module.scss");

    view! {
        <button on:click=on_click class=style::new_counter>
            New Counter
        </button>
    }
}
