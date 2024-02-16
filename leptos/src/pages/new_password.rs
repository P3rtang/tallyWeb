use leptos::*;
use leptos_router::{ActionForm, A};
use web_sys::SubmitEvent;

use super::*;
use components::MessageBox;

#[component]
pub fn NewPassword() -> impl IntoView {
    let user = expect_context::<RwSignal<UserSession>>();
    let message = expect_context::<MessageBox>();

    let action = create_server_action::<api::ChangePassword>();

    let new_pass_ref = create_node_ref::<leptos::html::Input>();
    let new_pass_repeat_ref = create_node_ref::<leptos::html::Input>();

    let on_submit = move |ev: SubmitEvent| {
        if new_pass_ref().unwrap().value() != new_pass_repeat_ref().unwrap().value() {
            message.set_err("Passwords do not match");
            ev.prevent_default();
        } else if new_pass_ref().unwrap().value().len() < 8 {
            message.set_err("Password should be longer than 8 characters");
            ev.prevent_default();
        }

        create_effect(move |_| {
            if let Some(Ok(_)) = action.value()() {
                message.set_msg("Password succesfully changed")
            } else if let Some(Err(_)) = action.value()() {
                message.set_err("An error occurred")
            }
        });
    };

    view! {
        <ActionForm action on:submit=on_submit>
            <div class="container login-form">
                <input type="hidden" name="username" value=move || user().username/>
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
                    node_ref=new_pass_ref
                    required
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
