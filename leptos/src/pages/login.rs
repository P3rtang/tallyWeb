use super::*;
use leptos::*;
use leptos_router::{ActionForm, A};

#[component]
pub fn LoginPage() -> impl IntoView {
    let login_action = create_server_action::<api::LoginUser>();

    let on_submit = move |_| {
        create_effect(move |_| {
            if login_action.value().get().is_some_and(|v| v.is_ok()) {
                let _ = window().location().set_href("/");
            }
        });
    };

    view! {
        <ActionForm action=login_action on:submit=on_submit>
            <div class="container login-form">
                <h1>Login</h1>
                <label for="username">
                    <b>Username</b>
                </label>
                <input
                    type="text"
                    placeholder="Enter Username"
                    name="username"
                    id="username"
                    autocomplete="username"
                    required
                />
                <label for="password">
                    <b>Password</b>
                </label>
                <input
                    type="password"
                    placeholder="Enter Password"
                    name="password"
                    id="password"
                    autocomplete="current-password"
                    required
                />

                <div class="action-buttons">
                    <div class="action-buttons-el">
                        <input type="checkbox" name="remember" id="remember"/>
                        <label for="remember">Remember Me</label>
                    </div>
                    <A href="/create-account">
                        <i class="fa-solid fa-user-plus"></i>
                    </A>
                    <button type="submit" name="Sign in">
                        <i class="fa-solid fa-right-to-bracket"></i>
                    </button>
                </div>
            </div>
        </ActionForm>
    }
}
