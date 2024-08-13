use super::*;
use components::MessageJar;
use leptos::*;
use leptos_router::{ActionForm, A};

#[component]
pub fn ChangeAccountInfo() -> impl IntoView {
    let message = expect_context::<MessageJar>();
    let user = expect_context::<RwSignal<UserSession>>();

    let action = create_server_action::<api::ServerChangeAccountInfo>();

    create_effect(move |_| match action.value().get() {
        Some(Ok(_)) => message.set_msg("Username succesfully changed"),
        Some(Err(err)) => message.set_err(AppError::from(err)),
        None => {}
    });

    view! {
        <ActionForm action>
            <div class="container login-form">
                <input type="hidden" name="old_username" value=move || user().username />
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
