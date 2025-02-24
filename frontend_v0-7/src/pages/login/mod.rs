use super::*;
use leptos::form::ActionForm;

stylance::import_style!(
    #[allow(dead_code)]
    style,
    "./login.module.scss"
);

#[component]
pub fn LoginPage() -> impl IntoView {
    let login_action = ServerAction::<api::LoginUser>::new();
    // let message_jar = expect_context::<components::MessageJar>();

    let server_resp = Memo::new(move |_| {
        if let Some(Err(_err)) = login_action.value().get() {
            // message_jar.set_err(AppError::from(err))
        }
    });

    Effect::new(move |_| server_resp.track());

    // TODO: reset indexed_db
    //
    // #[cfg(not(feature = "ssr"))]
    // leptos::task::spawn_local(async move {
    //     if let Err(err) = indexed::IndexedSaveHandler::reset().await {
    //         message_jar.set_err(err)
    //     }
    // });

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
                    <Button class=style::remember rounding=ButtonRounding::Full attr:r#type="button">
                        <input type="checkbox" name="remember" id="remember" />
                        <label for="remember">Remember Me</label>
                    </Button>
                    <Button
                        href="/create-account"
                        rounding=ButtonRounding::Full
                        size=ButtonSize::Big
                        attr:r#type="button"
                    >
                        <Icon color=IconColor::Black kind=IconKind::AddAccount />
                    </Button>
                    <Button
                        rounding=ButtonRounding::Full
                        size=ButtonSize::Big
                        attr:r#type="submit"
                        attr:aria-label="sign in button"
                    >
                        <Icon color=IconColor::Black kind=IconKind::LogIn />
                    </Button>
                </action-buttons>
            </div>
        </ActionForm>
    }
}
