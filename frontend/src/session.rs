#![allow(unused_braces)]
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
    match actix_extract_user().await {
        Ok(session) => session,
        Err(_) => UserSession::default(),
    }
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
            name="session_user_uuid"
            value=move || session_val.get_value()().user_uuid.to_string()
        />
        <input
            type="hidden"
            name="session_username"
            value=move || session_val.get_value()().username
        />
        <input
            type="hidden"
            name="session_token"
            value=move || session_val.get_value()().token.to_string()
        />
    }
}

pub async fn session_signal() -> Result<(), AppError> {
    let owner = Owner::current().unwrap();

    let user = RwSignal::new(get_user_signal().await);
    api::check_user(user.get()).await?;

    owner.with(move || provide_context(user));

    Ok(())
}
