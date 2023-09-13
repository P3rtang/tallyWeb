use gloo_storage::{LocalStorage, Storage};
use leptos::*;
use leptos_router::{ActionForm, FromFormData, A};
use web_sys::SubmitEvent;

use crate::app::{navigate, CreateAccount};

#[component]
pub fn CreateAccount(cx: Scope) -> impl IntoView {
    let action = create_server_action::<crate::app::CreateAccount>(cx);

    let (err_message, set_err_message) = create_signal(cx, None);

    let show_err = move || {
        if err_message().is_some() {
            "display: block; color: red;"
        } else {
            "display: none"
        }
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
        set_err_message.set(if let Some(Err(err)) = action.value().get() {
            Some(
                err.to_string()
                    .split_once(": ")
                    .map(|s| s.1.to_string())
                    .unwrap_or_default(),
            )
        } else {
            None
        })
    });

    let on_submit = move |ev: SubmitEvent| {
        if let Ok(data) = CreateAccount::from_event(&ev) {
            if data.password != data.password_repeat {
                set_err_message(Some(format!("passwords do not match")));
                ev.prevent_default();
            }
        }
    };

    view! { cx,
        <ActionForm action=action on:submit=on_submit>
            <div class="container login-form">
                <h1>Sign Up</h1>

                <label for="username"><b>Username</b></label>
                <input type="text" placeholder="Enter Username" name="username" required/>

                <label for="password"><b>Password</b></label>
                <input type="password" placeholder="Enter Password" name="password" required/>

                <label for="password_repeat"><b>Repeat Password</b></label>
                <input type="password" placeholder="Repeat Password" name="password_repeat" required/>

                // <p>By creating an account you agree to our <a href="#" style="color:dodgerblue">Terms & Privacy</a>.</p>

                <div class="clearfix action-buttons">
                    <div class="action-buttons-el">
                        <input type="checkbox" required></input>
                        <A href="/privacy-policy" class="acceptTS"><b>Terms & Conditions</b></A>
                    </div>
                    <button type="button" on:click=move |_| { navigate(cx, "/login") }><i class="fa-solid fa-xmark"></i></button>
                    <button type="submit" class="signupbtn"><i class="fa-solid fa-right-to-bracket"></i></button>
                </div>
            </div>
        </ActionForm>
        <label style=show_err class="notification-box">{ move || { err_message() } }</label>
    }
}
