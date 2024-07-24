use leptos::*;
use leptos_router::{ActionForm, A};

use super::*;
use components::MessageJar;

#[component]
pub fn ChangePassword() -> impl IntoView {
    let user = expect_context::<RwSignal<UserSession>>();
    let message = expect_context::<MessageJar>();

    let action = create_server_action::<api::ChangePassword>();

    let new_pass_ref = create_node_ref::<leptos::html::Input>();
    let new_pass_repeat_ref = create_node_ref::<leptos::html::Input>();

    let on_submit = move |ev: ev::SubmitEvent| {
        if new_pass_ref().unwrap().value() != new_pass_repeat_ref().unwrap().value() {
            message.set_err("Passwords do not match");
            ev.prevent_default();
        } else if new_pass_ref().unwrap().value().len() < 8 {
            message.set_err("Password should be longer than 8 characters");
            ev.prevent_default();
        }
    };

    create_effect(move |_| match action.value().get() {
        Some(Ok(_)) => message.set_msg("Password succesfully changed"),
        Some(Err(err)) => message.set_err(err.to_string()),
        None => {}
    });

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
