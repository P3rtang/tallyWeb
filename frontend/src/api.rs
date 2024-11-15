use countable::{self, Counter};
use leptos::*;

use super::*;

#[cfg(feature = "ssr")]
use actix_web::web::Data;
#[cfg(feature = "ssr")]
use leptos_actix::extract;

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

    let user = backend::auth::login_user(&pool, username, password, dur).await?;

    let session = UserSession {
        user_uuid: user.uuid,
        username: user.username,
        token: user.token.unwrap(),
    };

    set_session_cookie(session.clone()).await?;
    leptos_actix::redirect("/");

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

#[allow(clippy::too_many_arguments)]
#[server(EditCountableForm)]
pub async fn edit_countable_form(
    session_user_uuid: uuid::Uuid,
    session_username: String,
    session_token: uuid::Uuid,

    countable_key: uuid::Uuid,
    countable_kind: CountableKind,
    countable_name: String,
    countable_count: i32,
    countable_step: i32,
    countable_hours: i64,
    countable_mins: i64,
    countable_secs: i64,
    countable_millis: i64,
    countable_hunttype: String,
    countable_charm: Option<String>,
) -> Result<(), ServerFnError> {
    let session = UserSession {
        user_uuid: session_user_uuid,
        username: session_username,
        token: session_token,
    };
    check_user(session).await?;

    let countable_time =
        ((countable_hours * 60 + countable_mins) * 60 + countable_secs) * 1000 + countable_millis;

    let mut conn = extract_pool().await?.begin().await?;
    match countable_kind {
        CountableKind::Counter => {
            backend::counter::set_name(&mut conn, countable_key, &countable_name).await?;
            backend::counter::set_count(&mut conn, countable_key, countable_count).await?;
            backend::counter::set_step(&mut conn, countable_key, countable_step).await?;
            backend::counter::set_time(&mut conn, countable_key, countable_time).await?;
            backend::counter::set_hunttype(&mut conn, countable_key, countable_hunttype.into())
                .await?;
            backend::counter::set_charm(&mut conn, countable_key, countable_charm.is_some())
                .await?;
        }
        CountableKind::Phase => {
            backend::phase::set_name(&mut conn, countable_key, &countable_name).await?;
            backend::phase::set_count(&mut conn, countable_key, countable_count).await?;
            backend::phase::set_step(&mut conn, countable_key, countable_step).await?;
            backend::phase::set_time(&mut conn, countable_key, countable_time).await?;
            backend::phase::set_hunttype(&mut conn, countable_key, countable_hunttype.into())
                .await?;
            backend::phase::set_charm(&mut conn, countable_key, countable_charm.is_some()).await?;
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
                if c.lock()?.owner_uuid != session.user_uuid {
                    Err(AppError::Unauthorized)?
                }
                backend::counter::update(&mut tx, c.lock()?.clone().into()).await?
            }
            countable::Countable::Phase(p) => {
                if p.lock()?.owner_uuid != session.user_uuid {
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
    countable: Countable,
) -> Result<(), ServerFnError> {
    match countable {
        Countable::Counter(_) => remove_counter(session, countable.uuid()).await?,
        Countable::Phase(_) => remove_phase(session, countable.uuid()).await?,
        Countable::Chain(_) => todo!(),
    }

    return Ok(());
}

#[server(RemoveCounter, "/api")]
pub async fn remove_counter(
    session: UserSession,
    counter_id: uuid::Uuid,
) -> Result<(), ServerFnError> {
    let pool = extract_pool().await?;

    backend::remove_counter(&pool, &session.username, session.token, counter_id).await?;

    Ok(())
}

#[server(RemovePhase, "/api")]
pub async fn remove_phase(session: UserSession, phase_id: uuid::Uuid) -> Result<(), ServerFnError> {
    let pool = extract_pool().await?;

    let _ = backend::auth::get_user(&pool, &session.username, session.token).await?;
    backend::remove_phase(&pool, phase_id).await?;

    Ok(())
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
                session.user_uuid,
                session.username,
                session.token,
                new_prefs.clone(),
            )
            .await?;
            new_prefs
        }
        Err(err) => return Err(err)?,
    };

    Ok(Preferences::from(prefs))
}

#[server(SavePreferences, "/api/session")]
pub async fn save_preferences(
    session_user_uuid: uuid::Uuid,
    session_username: String,
    session_token: uuid::Uuid,
    preferences: Preferences,
) -> Result<(), ServerFnError> {
    let session = UserSession {
        user_uuid: session_user_uuid,
        username: session_username,
        token: session_token,
    };
    let pool = extract_pool().await?;

    let user = backend::auth::get_user(&pool, &session.username, session.token).await?;

    let accent_color = if preferences.use_default_accent_color {
        None
    } else {
        Some(preferences.accent_color.0)
    };

    let db_prefs = backend::DbPreferences {
        user_uuid: user.uuid,
        use_default_accent_color: preferences.use_default_accent_color,
        accent_color,
        show_separator: preferences.show_separator,
        multi_select: preferences.multi_select,
        save_on_pause: preferences.save_on_pause,
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
