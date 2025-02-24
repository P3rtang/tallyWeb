use leptos::{form::ActionForm, html::Input, prelude::*};
use leptos_router::hooks::use_navigate;
use web_sys::SubmitEvent;

use super::*;

stylance::import_style!(style, "../login/login.module.scss");

#[component]
pub fn CreateAccount() -> impl IntoView {
    let action = ServerAction::<api::CreateAccount>::new();
    // let message_jar = expect_context::<components::MessageJar>();

    let password_input = NodeRef::<Input>::new();
    let password_repeat = NodeRef::<Input>::new();

    let on_submit = move |ev: SubmitEvent| {
        if password_input.get().unwrap().value().len() < 8 {
            // message.set(Some(String::from(
            //     "Password should be longer than 8 characters",
            // )));
            ev.prevent_default()
        }
        if password_input.get().unwrap().value() != password_repeat.get().unwrap().value() {
            // message.set(Some(String::from("passwords do not match")));
            ev.prevent_default();
        }
        Effect::new(move |_| match action.value().get() {
            Some(Ok(_)) => use_navigate()("/", Default::default()),
            Some(Err(err)) => warn!("{}", err),
            None => {}
        });
    };

    view! {
        <ActionForm action on:submit=on_submit>
            <div class=style::login_form>
                <h1>Sign Up</h1>

                <label for="username">
                    <b>Username</b>
                </label>
                <input type="text" placeholder="Enter Username" name="username" required />

                <label for="password">
                    <b>Password</b>
                </label>
                <input
                    type="password"
                    placeholder="Enter Password"
                    name="password"
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
                    node_ref=password_repeat
                    required
                />

                <action-buttons>
                    <Button class=style::remember rounding=ButtonRounding::Full attr:r#type="button">
                        <input required type="checkbox" name="accept-tc" id="accept-tc" />
                        <label for="accept-tc" style:font-size="16px">{"I have read the Terms&Conditions"}</label>
                    </Button>
                    <Button
                        href="/login"
                        rounding=ButtonRounding::Full
                        size=ButtonSize::Big
                        attr:r#type="button"
                    >
                        <Icon color=IconColor::Black kind=IconKind::Cross />
                    </Button>
                    <Button
                        rounding=ButtonRounding::Full
                        size=ButtonSize::Big
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
