use super::*;
use components::{LoadingScreen, Message};
use gloo_storage::{LocalStorage, Storage};
use leptos::*;
use leptos_router::ActionForm;

#[server(ServerChangeAccountInfo, "/api")]
async fn change_username(
    old_username: String,
    password: String,
    new_username: String,
) -> Result<SessionUser, ServerFnError> {
    let pool = backend::create_pool().await?;
    let user = backend::auth::change_username(&pool, old_username, new_username, password).await?;

    let session_user = SessionUser {
        username: user.username,
        token: user.token.unwrap(),
    };

    return Ok(session_user);
}

#[component]
pub fn ChangeAccountInfo(user: RwSignal<Option<SessionUser>>) -> impl IntoView {
    let message = expect_context::<Message>();

    let action = create_server_action::<ServerChangeAccountInfo>();

    let on_submit = move |_| {
        create_effect(move |_| {
            if let Some(Ok(session_user)) = action.value()() {
                message.set_message("Username succesfully changed");
                if let Ok(_) = LocalStorage::set("user_session", session_user.clone()) {
                    expect_context::<RwSignal<Option<SessionUser>>>()
                        .set(Some(session_user.clone()));
                    user.set(Some(session_user));
                }
            } else if let Some(Err(err)) = action.value()() {
                let err_str = err.to_string();
                let err_msg = err_str.split(": ").last().unwrap();
                message.set_error(err_msg)
            }
        });
    };

    view! {
        <Show
            when=move || user().is_some()
            fallback=LoadingScreen
        >
            <ActionForm action on:submit=on_submit>
                <div class="container login-form">
                    <input type="hidden" name="old_username" value=move || user().unwrap().username/>
                    <input type="text" name="new_username" placeholder="New Username" value=move || user().unwrap().username/>
                    <input
                        type="password"
                        name="password"
                        id="password"
                        placeholder="Password"
                        required
                    />
                    <div class="clearfix action-buttons">
                        <button type="button" on:click=move |_| { navigate( "/preferences") }><i class="fa-solid fa-xmark"></i></button>
                        <button type="submit" class="signupbtn"><i class="fa-solid fa-right-to-bracket"></i></button>
                    </div>
                </div>
            </ActionForm>
        </Show>
    }
}
