use gloo_storage::{LocalStorage, Storage};
use leptos::*;
use leptos_router::ActionForm;

#[component]
pub fn CreateAccount(cx: Scope) -> impl IntoView {
    let action = create_server_action::<crate::app::CreateAccount>(cx);

    let show_err = move || {
        if let Some(func) = action.input().get() {
            if func.password != func.password_repeat {
                "display: block; color: red;"
            } else {
                "display: none"
            }
        } else {
            "display: none"
        }
    };

    create_effect(cx, move |_| {
        let _ = LocalStorage::set(
            "user_session",
            &action
                .value()
                .get()
                .map(|v| v.ok())
                .flatten()
                .unwrap_or_default(),
        );

        if action.value().get().is_some() && action.value().get().unwrap().is_ok() {
            crate::app::navigate(cx, "/")
        }
    });

    view! { cx,
        <ActionForm action=action>
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
            <label style=show_err>Invalid Login Information</label>
        </ActionForm>
    }
}
