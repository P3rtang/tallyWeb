#![allow(unused_braces)]
#![allow(dead_code)]

use super::*;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[server]
pub async fn actix_extract_user() -> Result<UserSession, ServerFnError> {
    use super::AppError;
    use leptos_actix::{extract, redirect};

    let header = extract::<actix_web::HttpRequest>().await?;
    match header.cookie("session") {
        Some(session) => return Ok(serde_json::from_str(&session.value().to_string())?),

        None => {
            redirect("/login");
            return Err(AppError::MissingSession)?;
        }
    };
}

async fn get_user_signal() -> UserSession {
    actix_extract_user().await.unwrap_or_default()
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserSession {
    pub user_uuid: uuid::Uuid,
    pub username: String,
    pub token: uuid::Uuid,
}

#[component(transparent)]
pub fn SessionFormInput(#[prop(into)] session: Signal<UserSession>) -> impl IntoView {
    let session_val = StoredValue::new(session);
    view! {
        <input
            type="hidden"
            name="session[user_uuid]"
            value=move || session_val.get_value()().user_uuid.to_string()
        />
        <input
            type="hidden"
            name="session[username]"
            value=move || session_val.get_value()().username
        />
        <input
            type="hidden"
            name="session[token]"
            value=move || session_val.get_value()().token.to_string()
        />
    }
}

pub fn provide_session() -> Resource<UserSession> {
    let owner = Owner::current().unwrap();

    let user = RwSignal::new(UserSession::default());
    let user_resc = Resource::new_blocking(
        move || (),
        move |_| async {
            let user = get_user_signal().await;
            // TODO: regenerate token when expired error
            let _ = api::check_user(user.clone()).await;
            user
        },
    );

    Effect::new_isomorphic(move || {
        if let Some(u) = user_resc.get() {
            user.set(u)
        }
    });

    owner.with(move || provide_context(user));

    return user_resc;
}
