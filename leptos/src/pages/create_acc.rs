use gloo_storage::{SessionStorage, Storage};
use leptos::*;
use leptos_router::{ActionForm, FromFormData, A};
use web_sys::SubmitEvent;

use crate::app::CreateAccount;

#[component]
pub fn CreateAccount(cx: Scope) -> impl IntoView {
    let action = create_server_action::<crate::app::CreateAccount>(cx);

    let (err_message, set_err_message) = create_signal(cx, None);

    let show_err = move || {
        if action.value().get().map(|v| v.ok()).flatten().is_none() || err_message().is_some() {
            "display: block; color: red;"
        } else {
            "display: none"
        }
    };

    create_effect(cx, move |_| {
        if let Some(login) = action.value().get().map(|v| v.ok()).flatten() {
            if let Ok(_) = SessionStorage::set("user_session", login.clone()) {
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
            Some(String::new())
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
            <div class="container">
                <h1>Sign Up</h1>
                <p>Please fill in this form to create an account.</p>
                <hr/>

                <label for="username"><b>Username</b></label>
                <input type="text" placeholder="Enter Username" name="username" required/>

                <label for="password"><b>Password</b></label>
                <input type="password" placeholder="Enter Password" name="password" required/>

                <label for="password_repeat"><b>Repeat Password</b></label>
                <input type="password" placeholder="Repeat Password" name="password_repeat" required/>

                // <p>By creating an account you agree to our <a href="#" style="color:dodgerblue">Terms & Privacy</a>.</p>

                <div class="clearfix">
                    <button type="button" class="cancelbtn">Cancel</button>
                    <button type="submit" class="signupbtn">Sign Up</button>
                </div>
            </div>
            <label style=show_err>{ move || { err_message() } }</label>
            <input type="checkbox" required></input>
        </ActionForm>
        <A href="/privacy-policy" class="acceptTS">Accept Terms & Service</A>
    }
}
