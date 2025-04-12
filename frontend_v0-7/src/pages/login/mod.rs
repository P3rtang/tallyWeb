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

    let message = use_message();

    Effect::new(move |_| {
        if let Some(Err(err)) = login_action.value().get() {
            message.server_err(err);
        }
    });

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
                    <Button
                        class=style::remember
                        attr:r#type="button"
                        attr:tabindex=-1
                        xstyle=xstyle!("border-radius": XBorderRadius::Full)
                    >
                        <input type="checkbox" name="remember" id="remember" />
                        <label for="remember">Remember Me</label>
                    </Button>
                    <Button
                        href="/create-account"
                        xstyle=xstyle!("padding": XPadding::Big, "border-radius": XBorderRadius::Full)
                        attr:r#type="button"
                    >
                        <Icon color=IconColor::Black kind=IconKind::AddAccount />
                    </Button>
                    <Button
                        xstyle=xstyle!("padding": XPadding::Big, "border-radius": XBorderRadius::Full)
                        attr:r#type="submit"
                        attr:aria-label="sign in button"
                        attr:label="sign-in"
                    >
                        <Icon color=IconColor::Black kind=IconKind::LogIn />
                    </Button>
                </action-buttons>
            </div>
        </ActionForm>
    }
}
