use gloo_storage::{SessionStorage, Storage};
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
        if let Some(login) = action.value().get().map(|v| v.ok()).flatten() {
            if let Ok(_) = SessionStorage::set("user_session", login.clone()) {
                if action.value().get().is_some() && action.value().get().unwrap().is_ok() {
                    crate::app::navigate(cx, "/")
                }
            }
        }
    });

    view! { cx,
        <ActionForm action=action>
        <div class="container">
            <label for="username"><b>Username</b></label>
                <input
                    type="text"
                    placeholder="Enter Username"
                    name="username"
                    id="username"
                    autocomplete="username"
                    required
                />
            <label for="password"><b>Password</b></label>
            <input
                type="password"
                placeholder="Enter Password"
                name="password"
                id="password"
                autocomplete="current-password"
                required
            />

            <button type="submit">Login</button>
        </div>
        </ActionForm>
        <p style=show_err>{ move || {
            if let Some(Err(err)) = action.value().get() {
                err.to_string().split_once(": ").map(|s| s.1.to_string()).unwrap_or_default()
            } else {
                String::new()
            }
        }}</p>
        <A href="/create-account">Create New Account</A>
    }
}
