use super::*;

stylance::import_style!(style, "../login/login.module.scss");

#[component]
pub fn CreateAccount() -> impl IntoView {
    let action = ServerAction::<api::CreateAccount>::new();
    let message = use_message();

    let password_input = NodeRef::<Input>::new();
    let password_repeat = NodeRef::<Input>::new();

    Effect::new(move |_| match action.value().get() {
        Some(Ok(_)) => use_navigate()("/", Default::default()),
        Some(Err(err)) => {
            message.server_err(err);
            action.value().update_untracked(|v| {
                v.take();
            });
        }
        None => {}
    });

    view! {
        <ActionForm action>
            <div class=style::login_form>
                <h1>Sign Up</h1>
                <label for="username">
                    <b>Username</b>
                </label>
                <input
                    type="text"
                    placeholder="Enter Username"
                    name="username"
                    id="username"
                    autocomplete="off"
                    required
                />
                <label for="password">
                    <b>Password</b>
                </label>
                <input
                    type="password"
                    placeholder="Enter Password"
                    name="password"
                    id="password"
                    node_ref=password_input
                    required
                />

                <label for="password_repeat">
                    <b>Repeat Password</b>
                </label>
                <input
                    type="password"
                    placeholder="Repeat Password"
                    name="password_repeat"
                    id="password_repeat"
                    node_ref=password_repeat
                    required
                />

                <action-buttons>
                    <Button
                        xstyle=xstyle!("border-radius": XBorderRadius::Full)
                        class=style::remember
                        attr:tabindex=-1
                        attr:r#type="button"
                    >
                        <input required type="checkbox" name="accept-tc" id="accept-tc" />
                        <label for="accept-tc" style:font-size="16px">
                            <a href="/terms">
                            {"I have read the Terms&Conditions"}
                            </a>
                        </label>
                    </Button>
                    <Button
                        href="/login"
                        xstyle=xstyle!("padding": XPadding::Big, "border-radius": XBorderRadius::Full)
                        attr:r#type="button"
                    >
                        <Icon color=IconColor::Black kind=IconKind::Cross />
                    </Button>
                    <Button
                        xstyle=xstyle!("padding": XPadding::Big, "border-radius": XBorderRadius::Full)
                        attr:r#type="submit"
                        attr:aria-label="sign up button"
                    >
                        <Icon color=IconColor::Black kind=IconKind::LogIn />
                    </Button>
                </action-buttons>
            </div>
        </ActionForm>
    }
}
