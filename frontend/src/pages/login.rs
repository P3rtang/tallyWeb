use super::*;
use leptos::*;
use leptos_router::{ActionForm, A};

stylance::import_style!(
    #[allow(dead_code)]
    style,
    "../../style/login.module.scss"
);

#[component]
pub fn LoginPage() -> impl IntoView {
    let login_action = create_server_action::<api::LoginUser>();
    let message_jar = expect_context::<components::MessageJar>();

    let server_resp = create_memo(move |_| {
        if let Some(Err(err)) = login_action.value().get() {
            message_jar.set_err(AppError::from(err))
        }
    });

    create_effect(move |_| server_resp.track());

    view! {
        <ActionForm action=login_action>
            <div class=style::login_form>
                <h1>Login</h1>
                <label for="username">Username</label>
                <input
                    type="text"
                    placeholder="Enter Username"
                    name="username"
                    id="username"
                    autocomplete="username"
                    required
                />
                <label for="password">Password</label>
                <input
                    type="password"
                    placeholder="Enter Password"
                    name="password"
                    id="password"
                    autocomplete="current-password"
                    required
                />

                <action-buttons>
                    <div class=style::action_button_el>
                        <input type="checkbox" name="remember" id="remember" />
                        <label for="remember">Remember Me</label>
                    </div>
                    <A href="/create-account">
                        <i class="fa-solid fa-user-plus"></i>
                    </A>
                    <button type="submit" aria-label="button-sign-in">
                        <i class="fa-solid fa-right-to-bracket"></i>
                    </button>
                </action-buttons>
            </div>
        </ActionForm>
    }
}
