use gloo_storage::{LocalStorage, Storage};
use leptos::{html::Input, *};
use leptos_router::{ActionForm, A};
use web_sys::SubmitEvent;

use crate::app::navigate;

#[component]
pub fn CreateAccount(cx: Scope) -> impl IntoView {
    let action = create_server_action::<crate::app::CreateAccount>(cx);

    let message = create_rw_signal(cx, None::<String>);
    let border_style = move || {
        "color: tomato;
        border: 2px solid tomato;"
    };

    create_effect(cx, move |_| {
        if let Some(login) = action.value().get().map(|v| v.ok()).flatten() {
            if let Ok(_) = LocalStorage::set("user_session", login.clone()) {
                if action.value().get().is_some() && action.value().get().unwrap().is_ok() {
                    crate::app::navigate(cx, "/")
                }
            }
        }
    });

    create_effect(cx, move |_| {
        if let Some(Err(err)) = action.value().get() {
            message.set(err.to_string().split_once(": ").map(|s| s.1.to_string()))
        }
    });

    let password_input = create_node_ref::<Input>(cx);
    let password_repeat = create_node_ref::<Input>(cx);

    let on_submit = move |ev: SubmitEvent| {
        if password_input().unwrap().value().len() < 8 {
            message.set(Some(String::from(
                "Password should be longer than 8 characters",
            )));
            ev.prevent_default()
        }
        if password_input().unwrap().value() != password_repeat().unwrap().value() {
            message.set(Some(format!("passwords do not match")));
            ev.prevent_default();
        }
    };

    view! { cx,
        <ActionForm action=action on:submit=on_submit>
            <div class="container login-form">
                <h1>Sign Up</h1>

                <label for="username"><b>Username</b></label>
                <input type="text" placeholder="Enter Username" name="username" required/>

                <label for="password"><b>Password</b></label>
                <input
                    type="password"
                    placeholder="Enter Password"
                    name="password"
                    node_ref=password_input
                    required
                />

                <label for="password_repeat"><b>Repeat Password</b></label>
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
                        <input type="checkbox" required></input>
                        <A href="/privacy-policy" class="acceptTS"><b>I have read the</b><b>Terms & Conditions</b></A>
                    </div>
                    <button type="button" on:click=move |_| { navigate(cx, "/login") }><i class="fa-solid fa-xmark"></i></button>
                    <button type="submit" class="signupbtn"><i class="fa-solid fa-right-to-bracket"></i></button>
                </div>
            </div>
        </ActionForm>
        <Show
            when=move || { message().is_some() }
            fallback=|_| ()
        >
            <b class="notification-box" style=border_style>{ move || { message().unwrap() } }</b>
        </Show>
    }
}
