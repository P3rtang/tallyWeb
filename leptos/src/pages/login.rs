use crate::app::{navigate, SessionUser};
use gloo_storage::{LocalStorage, Storage};
use leptos::{html::Input, *};
use leptos_router::ActionForm;

#[component]
pub fn LoginPage() -> impl IntoView {
    let action = create_server_action::<crate::app::LoginUser>();

    expect_context::<RwSignal<Option<SessionUser>>>().set(None);
    create_render_effect(move |_| {
        LocalStorage::set("user_session", None::<SessionUser>).unwrap();
    });

    let message = create_rw_signal(None::<String>);
    let border_style = move || {
        "color: tomato;
        border: 2px solid tomato;"
    };

    create_effect(move |_| {
        if let Some(login) = action.value().get().map(|v| v.ok()).flatten() {
            if let Ok(_) = LocalStorage::set("user_session", login.clone()) {
                expect_context::<RwSignal<Option<SessionUser>>>().set(Some(login));
                crate::app::navigate("/")
            }
        } else if let Some(Err(err)) = action.value().get() {
            message.set(err.to_string().split_once(": ").map(|s| s.1.to_string()));
        }
    });

    let password_input = create_node_ref::<Input>();

    view! {
        <ActionForm action=action on:submit=|_|()>
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
                    node_ref=password_input
                    required
                />

                <div class="action-buttons">
                <button type="button" on:click= move |_| navigate( "/create-account")><i class="fa-solid fa-user-plus"></i></button>
                    <button type="submit"><i class="fa-solid fa-right-to-bracket"></i></button>
                </div>
            </div>
        </ActionForm>
        <Show
            when=move || { message().is_some() }
            fallback=|| ()
        >
            <b class="notification-box" style=border_style>{ move || { message().unwrap() } }</b>
        </Show>
    }
}
