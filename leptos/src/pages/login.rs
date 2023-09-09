use gloo_storage::{LocalStorage, Storage};
use leptos::*;
use leptos_router::{ActionForm, A};

#[component]
pub fn LoginPage(cx: Scope) -> impl IntoView {
    let action = create_server_action::<crate::app::LoginUser>(cx);

    let show_err = move || {
        action.value().with(|v| {
            if let Some(user) = v && user.is_err() {
                "display: block; color: red;"
            } else {
                "display: none"
            }
        })
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
            <label for="username"><b>Username</b></label>
                <input type="text" placeholder="Enter Username" name="username" required/>
            <label for="password"><b>Password</b></label>
            <input type="password" placeholder="Enter Password" name="password" required/>

            <button type="submit">Login</button>
            <label style=show_err>Invalid Login Information</label>
        </div>
        </ActionForm>
        <A href="/create_account">Create New Account</A>
    }
}
