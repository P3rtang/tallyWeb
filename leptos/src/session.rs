#![allow(unused_braces)]
use super::*;
use leptos::*;
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

async fn get_user_signal() -> RwSignal<UserSession> {
    match actix_extract_user().await {
        Ok(session) => session.into(),

        Err(err) => {
            eprintln!("{err}");
            UserSession::default().into()
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserSession {
    pub user_uuid: uuid::Uuid,
    pub username: String,
    pub token: uuid::Uuid,
}

#[component(transparent)]
pub fn ProvideSessionSignal(children: ChildrenFn) -> impl IntoView {
    view! {
        <Await future=get_user_signal let:user>

            {
                create_blocking_resource(*user, move |u| { api::check_user(u) });
                provide_context(*user);
                children()
            }

        </Await>
    }
}
