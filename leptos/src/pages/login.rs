use crate::app::{navigate, SessionUser};
use gloo_storage::{LocalStorage, Storage};
use leptos::*;
use leptos_router::ActionForm;

#[component]
pub fn LoginPage(cx: Scope) -> impl IntoView {
    let action = create_server_action::<crate::app::LoginUser>(cx);
    create_effect(cx, move |_| {
        request_animation_frame(move || {
            expect_context::<RwSignal<Option<SessionUser>>>(cx).set(None);
            let _ = LocalStorage::set("user_session", None::<SessionUser>);
        })
    });

    let show_err = move || {
        action.value().with(|v| {
            if let Some(user) = v && user.is_err() {
                "display: block;
                color: tomato;
                border: 2px solid tomato;"
            } else {
                "display: none"
            }
        })
    };

    create_effect(cx, move |_| {
        if let Some(login) = action.value().get().map(|v| v.ok()).flatten() {
            if let Ok(_) = LocalStorage::set("user_session", login.clone()) {
                crate::app::navigate(cx, "/")
            }
        }
    });

    view! { cx,
        <ActionForm action=action>
        <div class="container login-form">
            <h1>Login</h1>
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

            <div class="action-buttons">
            <button type="button" on:click= move |_| navigate(cx, "/create-account")><i class="fa-solid fa-user-plus"></i></button>
                <button type="submit"><i class="fa-solid fa-right-to-bracket"></i></button>
            </div>
        </div>
        </ActionForm>
        <b style=show_err class="notification-box">{ move || {
            if let Some(Err(err)) = action.value().get() {
                err.to_string().split_once(": ").map(|s| s.1.to_string()).unwrap_or_default()
            } else {
                String::new()
            }
        }}</b>
    }
}
