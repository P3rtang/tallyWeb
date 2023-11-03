use crate::app::*;
use components::{Message, Spinner};
use leptos::*;

#[server(AddCounter, "/api")]
async fn add_counter(username: String, token: String, name: String) -> Result<(), ServerFnError> {
    let counter_id = create_counter(username.clone(), token.clone(), name.clone()).await?;
    let phase_id = create_phase(
        username.clone(),
        token.clone(),
        String::from("Phase 1"),
        crate::countable::Hunttype::NewOdds,
    )
    .await?;
    assign_phase(username, token, counter_id, phase_id).await?;

    return Ok(());
}

#[component]
pub fn NewCounterButton(state_len: Signal<usize>) -> impl IntoView {
    let user = expect_context::<Memo<Option<SessionUser>>>();
    let message = expect_context::<Message>();

    let add_counter_action = create_action(move |(user, counter_name): &(SessionUser, String)| {
        add_counter(
            user.username.clone(),
            user.token.clone(),
            counter_name.to_string(),
        )
    });

    create_effect(move |_| {
        match add_counter_action.value()() {
            Some(Ok(_)) => {
                expect_context::<StateResource>().refetch();
                message.clear();
            }
            Some(Err(err)) => message.set_server_err(&err.to_string()),
            None => {}
        };
    });

    let on_click = move |_| {
        if user.get_untracked().is_some() {
            add_counter_action.dispatch((
                user.get_untracked().unwrap(),
                format!("Counter {}", state_len() + 1),
            ));

            message.set_msg_view(view! {
                <div style="display: flex; align-items: center;">
                    <Spinner/>
                    <b style="font-size: 20px; padding-left: 24px;">Creating Counter</b>
                </div>
            })
        } else {
            message.set_msg("Login to create a new counter\nOffline support coming...")
        }
    };

    view! {
        <button
            on:click=on_click
            class="new-counter"
        >
            New Counter
        </button>
    }
}
