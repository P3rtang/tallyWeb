use leptos::*;
use leptos_router::ActionForm;
use web_sys::SubmitEvent;

use super::*;
use crate::elements::{LoadingScreen, Message};

#[component]
pub fn NewPassword() -> impl IntoView {
    let user = expect_context::<Memo<Option<SessionUser>>>();
    let message = expect_context::<Message>();

    let action = create_server_action::<ChangePassword>();

    let new_pass_ref = create_node_ref::<leptos::html::Input>();
    let new_pass_repeat_ref = create_node_ref::<leptos::html::Input>();

    let on_submit = move |ev: SubmitEvent| {
        if new_pass_ref().unwrap().value() != new_pass_repeat_ref().unwrap().value() {
            message.set_error("Passwords do not match");
            ev.prevent_default();
        } else if new_pass_ref().unwrap().value().len() < 8 {
            message.set_error("Password should be longer than 8 characters");
            ev.prevent_default();
        }

        create_effect(move |_| {
            if let Some(Ok(_)) = action.value()() {
                message.set_message("Password succesfully changed")
            } else if let Some(Err(_)) = action.value()() {
                message.set_error("An error occurred")
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
                    <input type="hidden" name="username" value=move || user().unwrap().username/>
                    <input
                        type="password"
                        name="old_pass"
                        id="old_pass"
                        placeholder="Old Password"
                        required
                    />
                    <input
                        type="password"
                        name="new_pass"
                        id="new_pass"
                        placeholder="New Password"
                        node_ref=new_pass_ref required
                    />
                    <input
                        type="password"
                        name="new_pass_repeat"
                        id="new_pass_repeat"
                        placeholder="Repeat Password"
                        node_ref=new_pass_repeat_ref
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
