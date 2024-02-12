use leptos::{html::Input, *};
use leptos_router::{ActionForm, A};
use web_sys::SubmitEvent;

use super::*;

#[component]
pub fn CreateAccount() -> impl IntoView {
    let action = create_server_action::<api::CreateAccount>();

    let password_input = create_node_ref::<Input>();
    let password_repeat = create_node_ref::<Input>();

    let on_submit = move |ev: SubmitEvent| {
        if password_input().unwrap().value().len() < 8 {
            // message.set(Some(String::from(
            //     "Password should be longer than 8 characters",
            // )));
            ev.prevent_default()
        }
        if password_input().unwrap().value() != password_repeat().unwrap().value() {
            // message.set(Some(String::from("passwords do not match")));
            ev.prevent_default();
        }
        create_effect(move |_| {
            if action.value().get().is_some_and(|v| v.is_ok()) {
                let _ = window().location().set_href("/");
            }
        });
    };

    view! {
        <ActionForm action=action on:submit=on_submit>
            <div class="container login-form">
                <h1>Sign Up</h1>

                <label for="username">
                    <b>Username</b>
                </label>
                <input type="text" placeholder="Enter Username" name="username" required/>

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

                // <p>By creating an account you agree to our <a href="#" style="color:dodgerblue">Terms & Privacy</a>.</p>

                <div class="clearfix action-buttons">
                    <div class="action-buttons-el">
                        <input type="checkbox" required/>
                        <A href="/privacy-policy" class="acceptTS">
                            <b>I have read the</b>
                            <b>Terms & Conditions</b>
                        </A>
                    </div>
                    <A href="/login">
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
