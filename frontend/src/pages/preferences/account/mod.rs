use super::*;

use leptos::prelude::Set as lSet;
use reactive_graph::traits::Set;

#[component]
pub fn AccountPreferences() -> impl IntoView {
    let message = use_message();
    let session = expect_context::<RwSignal<UserSession>>();
    let action = ServerAction::<api::account::Update>::new();
    let (_, set_cookie) = use_cookie::<UserSession, JsonSerdeCodec>("session");

    Effect::new(move |_| match action.value().get() {
        Some(Ok(s)) => {
            session.set(s.clone());
            set_cookie.set(Some(s))
        }
        Some(Err(err)) => {
            message.server_err(err);
        }
        None => (),
    });

    view! {
        <Form session action>
            <TextField id="new_username" label="New username" attr:name="data[new_username]" />
            <TextField
                id="password"
                label="Password"
                attr:r#type="password"
                attr:name="data[password]"
            />

            <label for="nav-change-pass">Change password</label>
            <Button
                href="change-password"
                attr:id="nav-change-pass"
                xstyle=xstyle!(
                    "padding": XPadding::Square(12),
                    "border-radius": XBorderRadius::Percentage(100)
                )
                style:grid-column="2"
                style:justify-self="start"
            >
                <Icon kind=IconKind::ArrowRight />
            </Button>
        </Form>
    }
}
