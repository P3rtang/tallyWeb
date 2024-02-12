use super::*;
use components::Message;
use gloo_storage::{LocalStorage, Storage};
use leptos::*;
use leptos_router::{ActionForm, A};

#[server(ServerChangeAccountInfo, "/api")]
async fn change_username(
    old_username: String,
    password: String,
    new_username: String,
) -> Result<UserSession, ServerFnError> {
    let pool = backend::create_pool().await?;
    let user = backend::auth::change_username(&pool, old_username, new_username, password).await?;

    let session_user = UserSession {
        user_uuid: user.uuid,
        username: user.username,
        token: user.token.unwrap(),
    };

    leptos_actix::redirect("/preferences");

    return Ok(session_user);
}

#[component]
pub fn ChangeAccountInfo() -> impl IntoView {
    let message = expect_context::<Message>();
    let user = expect_context::<RwSignal<UserSession>>();

    let action = create_server_action::<ServerChangeAccountInfo>();

    let on_submit = move |_| {
        create_effect(move |_| {
            if let Some(Ok(session_user)) = action.value()() {
                message.set_msg("Username succesfully changed");
                if LocalStorage::set("user_session", session_user.clone()).is_ok() {
                    user.set(session_user);
                }
            } else if let Some(Err(err)) = action.value()() {
                let err_str = err.to_string();
                let err_msg = err_str.split(": ").last().unwrap();
                message.set_err(err_msg)
            }
        });
    };

    view! {
        <ActionForm action on:submit=on_submit>
            <div class="container login-form">
                <input type="hidden" name="old_username" value=move || user().username/>
                <input
                    type="text"
                    name="new_username"
                    placeholder="New Username"
                    value=move || user().username
                />
                <input
                    type="password"
                    name="password"
                    id="password"
                    placeholder="Password"
                    required
                />
                <div class="clearfix action-buttons">
                    <A href="/preferences">
                        <i class="fa-solid fa-xmark"></i>
                    </A>
                    <button type="submit" class="signupbtn">
                        <i class="fa-solid fa-right-to-bracket"></i>
                    </button>
                </div>
            </div>
        </ActionForm>
    }
}
