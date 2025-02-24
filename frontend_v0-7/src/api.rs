use countable::{self, Counter};
use leptos::prelude::*;

use super::*;

#[cfg(feature = "ssr")]
mod ssr_import {
    use super::*;

    pub(crate) use actix_web::web::Data;
    pub(crate) use leptos_actix::extract;
    pub(crate) use session::actix_extract_user;
}

#[cfg(feature = "ssr")]
use ssr_import::*;

#[cfg(feature = "ssr")]
pub async fn extract_pool() -> Result<Data<backend::PgPool>, AppError> {
    extract::<Data<backend::PgPool>>()
        .await
        .map_err(|err| AppError::Extraction(err.to_string()))
}

#[server(CheckUser, "/api")]
pub async fn check_user(session: UserSession) -> Result<(), ServerFnError> {
    use backend::auth::SessionState;
    let pool = extract_pool().await?;
    match backend::auth::check_user(&pool, &session.username, session.token).await {
        Ok(SessionState::Valid) => Ok(()),
        Ok(SessionState::Expired) => Err(AppError::ExpiredToken)?,
        Err(err) => {
            leptos_actix::redirect("/login");
            Err(err.into())
        }
    }
}

#[server(LoginUser, "/api", "Url", "login_user")]
pub async fn login_user(
    username: String,
    password: String,
    remember: Option<String>,
) -> Result<UserSession, ServerFnError> {
    let pool = extract_pool().await?;

    let dur = if remember.is_some() {
        chrono::Duration::days(30)
    } else {
        chrono::Duration::days(1)
    };

    let user = backend::auth::login_user(&pool, username.clone(), password, dur).await?;

    let session = UserSession {
        user_uuid: user.uuid,
        username: user.username,
        token: user.token.unwrap(),
    };

    set_session_cookie(session.clone()).await?;
    leptos_actix::redirect(&format!("/{}", &username));

    Ok(session)
}

#[server(CreateAccount, "/api", "Url", "create_account")]
pub async fn create_account(
    username: String,
    password: String,
    password_repeat: String,
) -> Result<UserSession, ServerFnError> {
    if password != password_repeat {
        Err(backend::LoginError::InvalidPassword)?;
    }

    let pool = extract_pool().await?;
    let user = backend::auth::insert_user(&pool, &username, &password).await?;

    let session_user = UserSession {
        user_uuid: user.uuid,
        username: user.username,
        token: user.token.unwrap(),
    };

    login_user(username, password, None).await?;

    Ok(session_user)
}

#[server(ChangePassword, "/api")]
pub async fn change_password(
    username: String,
    old_pass: String,
    new_pass: String,
    new_pass_repeat: String,
) -> Result<(), ServerFnError> {
    if new_pass != new_pass_repeat {
        Err(backend::LoginError::InvalidPassword)?;
    };

    let pool = extract_pool().await?;
    let _ = backend::auth::change_password(&pool, username, old_pass, new_pass).await?;

    Ok(())
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
struct FormCountable {
    key: uuid::Uuid,
    kind: CountableKind,
    name: String,
    count: i32,
    step: i32,
    time: i64,
    hunttype: String,
    charm: Option<String>,
}

#[allow(clippy::too_many_arguments)]
#[server(EditCountableForm)]
pub async fn edit_countable_form(
    session: UserSession,
    countable: FormCountable,
) -> Result<(), ServerFnError> {
    check_user(session).await?;

    let mut conn = extract_pool().await?.begin().await?;
    match countable.kind {
        CountableKind::Counter => {
            backend::counter::set_name(&mut conn, countable.key, &countable.name).await?;
            backend::counter::set_count(&mut conn, countable.key, countable.count).await?;
            backend::counter::set_step(&mut conn, countable.key, countable.step).await?;
            backend::counter::set_time(&mut conn, countable.key, countable.time).await?;
            backend::counter::set_hunttype(&mut conn, countable.key, countable.hunttype.into())
                .await?;
            backend::counter::set_charm(&mut conn, countable.key, countable.charm.is_some())
                .await?;
        }
        CountableKind::Phase => {
            backend::phase::set_name(&mut conn, countable.key, &countable.name).await?;
            backend::phase::set_count(&mut conn, countable.key, countable.count).await?;
            backend::phase::set_step(&mut conn, countable.key, countable.step).await?;
            backend::phase::set_time(&mut conn, countable.key, countable.time).await?;
            backend::phase::set_hunttype(&mut conn, countable.key, countable.hunttype.into())
                .await?;
            backend::phase::set_charm(&mut conn, countable.key, countable.charm.is_some()).await?;
        }
        _ => (),
    }

    conn.commit().await?;

    return Ok(());
}

#[server(UpdateCountable, "/api/session")]
pub async fn update_countable_many(list: Vec<countable::Countable>) -> Result<(), ServerFnError> {
    let pool = extract_pool().await?;
    let session = session::actix_extract_user().await?;

    let mut tx = pool.begin().await?;

    for countable in list {
        match countable {
            countable::Countable::Counter(c) => {
                if c.lock()?.owner_uuid() != session.user_uuid {
                    Err(AppError::Unauthorized)?
                }
                backend::counter::update(&mut tx, c.lock()?.clone().into()).await?
            }
            countable::Countable::Phase(p) => {
                if p.lock()?.owner_uuid() != session.user_uuid {
                    Err(AppError::Unauthorized)?
                }
                backend::phase::update(&mut tx, p.lock()?.clone().into()).await?
            }
            countable::Countable::Chain(_) => todo!(),
        }
    }

    tx.commit().await?;

    return Ok(());
}

#[server(UpdateCounter, "/api")]
pub async fn update_counter(session: UserSession, counter: Counter) -> Result<(), ServerFnError> {
    let pool = extract_pool().await?;

    let _ =
        backend::update_counter(&pool, &session.username, session.token, counter.into()).await?;

    Ok(())
}

#[server(ArchiveCountable, "/api/session")]
pub async fn archive_countable(countable: Countable) -> Result<(), ServerFnError> {
    let pool = extract_pool().await?;
    let mut tx = pool.begin().await?;

    let uuid = countable.uuid();

    if let Err(err) = match countable {
        Countable::Counter(_) => backend::counter::archive(&mut tx, uuid).await,
        Countable::Phase(_) => backend::phase::archive(&mut tx, uuid).await,
        Countable::Chain(_) => todo!(),
    } {
        tx.rollback().await?;
        return Err(err.into());
    }

    tx.commit().await?;

    Ok(())
}

#[server(RemoveCountable, "/api/session")]
pub async fn remove_countable(
    session: UserSession,
    id: uuid::Uuid,
    kind: CountableKind,
) -> Result<Vec<uuid::Uuid>, ServerFnError> {
    // TODO: readd session checking
    let mut tx = extract_pool().await?.begin().await?;

    let deleted = match kind {
        CountableKind::Counter => backend::counter::remove(&mut tx, id).await?,
        CountableKind::Phase => vec![backend::phase::remove(&mut tx, id).await?],
        CountableKind::Chain => todo!(),
    };

    tx.commit().await?;

    leptos_actix::redirect(&format!("/{}", session.username));

    return Ok(deleted);
}

#[server(GetUserPreferences, "/api")]
pub async fn get_user_preferences(session: UserSession) -> Result<Preferences, ServerFnError> {
    let pool = extract_pool().await?;

    let user = match backend::auth::get_user(&pool, &session.username, session.token).await {
        Ok(user) => user,
        Err(_) => {
            return Ok(Preferences::default());
        }
    };
    let session_user = UserSession {
        user_uuid: user.uuid,
        username: user.username,
        token: user.token.unwrap_or_default(),
    };
    let prefs = match backend::DbPreferences::db_get(&pool, user.uuid).await {
        Ok(data) => Preferences::from_db(&session_user, data),
        Err(backend::BackendError::DataNotFound(_)) => {
            let new_prefs = Preferences::new(&session_user);
            save_preferences(
                session,
                FormPrefs {
                    use_default_accent_color: new_prefs
                        .use_default_accent_color
                        .then_some("on".into()),
                    accent_color: None,
                    show_body_border: new_prefs.show_body_border.then_some("on".into()),
                    show_separator: new_prefs.show_separator.then_some("on".into()),
                    save_on_pause: new_prefs.save_on_pause.then_some("on".into()),
                },
            )
            .await?;
            new_prefs
        }
        Err(err) => return Err(err)?,
    };

    Ok(Preferences::from(prefs))
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
struct FormPrefs {
    pub use_default_accent_color: Option<String>,
    pub accent_color: Option<String>,
    pub show_body_border: Option<String>,

    pub show_separator: Option<String>,
    pub save_on_pause: Option<String>,
}

#[server(SavePreferences, "/api/session")]
pub async fn save_preferences(
    session: UserSession,
    preferences: FormPrefs,
) -> Result<(), ServerFnError> {
    let pool = extract_pool().await?;

    let user = backend::auth::get_user(&pool, &session.username, session.token).await?;

    let db_prefs = backend::DbPreferences {
        user_uuid: user.uuid,
        use_default_accent_color: preferences.use_default_accent_color.is_some(),
        accent_color: preferences
            .use_default_accent_color
            .is_none()
            .then_some(preferences.accent_color)
            .flatten(),
        show_separator: preferences.show_separator.is_some(),
        multi_select: true,
        save_on_pause: preferences.save_on_pause.is_some(),
        show_body_border: preferences.show_body_border.is_some(),
    };
    db_prefs
        .db_set(&pool, &session.username, session.token)
        .await?;

    Ok(())
}

#[server]
pub async fn set_session_cookie(session: UserSession) -> Result<(), ServerFnError> {
    use actix_web::cookie::{self, time::Duration};
    use actix_web::http::header;

    let resp = expect_context::<leptos_actix::ResponseOptions>();

    let mut cookie = cookie::Cookie::new("session", serde_json::to_string(&session)?);
    cookie.set_max_age(Duration::days(30));
    cookie.set_path("/");

    resp.append_header(
        header::SET_COOKIE,
        header::HeaderValue::from_str(&cookie.to_string())?,
    );

    return Ok(());
}

#[server(ServerChangeAccountInfo, "/api")]
async fn change_username(
    old_username: String,
    password: String,
    new_username: String,
) -> Result<UserSession, ServerFnError> {
    let pool = api::extract_pool().await?;
    let user =
        backend::auth::change_username(&pool, &old_username, &new_username, &password).await?;

    let session_user = UserSession {
        user_uuid: user.uuid,
        username: user.username.clone(),
        token: user.token.unwrap(),
    };

    api::login_user(user.username, password, Some(String::new())).await?;
    leptos_actix::redirect("/preferences");

    return Ok(session_user);
}

#[server(CreateCountable, "/session/api")]
pub async fn create_countable(
    kind: CountableKind,
    parent: Option<uuid::Uuid>,
) -> Result<Vec<Countable>, ServerFnError> {
    let mut conn = extract_pool().await?.begin().await?;
    let user = actix_extract_user().await?;

    let countable: Vec<Countable> = match kind {
        CountableKind::Counter => {
            let counter_len = backend::counter::all_by_user(&mut conn, user.user_uuid)
                .await?
                .len();
            let (counter, phase) = backend::counter::create(
                &mut conn,
                user.user_uuid,
                format!("Counter {}", counter_len + 1),
            )
            .await?;
            vec![counter.into(), phase.into()]
        }
        CountableKind::Phase => {
            let parent = parent.ok_or(AppError::MissingParent)?;
            let phase_len = backend::counter::get_children(&mut conn, parent)
                .await?
                .len();
            let id = backend::phase::create(
                &mut conn,
                user.user_uuid,
                parent,
                format!("Phase {}", phase_len + 1),
            )
            .await?;
            vec![id.into()]
        }
        CountableKind::Chain => todo!(),
    };

    conn.commit().await?;

    return Ok(countable);
}
