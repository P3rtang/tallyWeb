use super::*;
use leptos::*;
use serde::{Deserialize, Serialize};

#[cfg(feature = "ssr")]
use actix_web::web::Data;
#[cfg(feature = "ssr")]
use leptos_actix::extract;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Countable {
    Counter(SerCounter),
    Phase(Phase),
}

impl From<Countable> for ArcCountable {
    fn from(val: Countable) -> Self {
        match val {
            Countable::Counter(c) => ArcCountable::new(Box::new(c)),
            Countable::Phase(p) => ArcCountable::new(Box::new(p)),
        }
    }
}

#[cfg(feature = "ssr")]
pub async fn extract_pool() -> Result<Data<backend::PgPool>, AppError> {
    extract::<Data<backend::PgPool>>()
        .await
        .map_err(|err| AppError::Extraction(err.to_string()))
}

#[server(CheckUser, "/api")]
pub async fn check_user(session: UserSession) -> Result<(), ServerFnError> {
    let pool = extract_pool().await?;
    match backend::auth::check_user(&pool, &session.username, session.token).await {
        Ok(()) => return Ok(()),
        Err(err) => {
            leptos_actix::redirect("/login");
            return Err(err)?;
        }
    }
}

#[server(LoginUser, "/api", "Url", "login_user")]
pub async fn login_user(username: String, password: String) -> Result<UserSession, ServerFnError> {
    let pool = extract_pool().await?;
    let user = backend::auth::login_user(&pool, username, password).await?;

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

    login_user(username, password).await?;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CounterResponse {
    Counters(Vec<SerCounter>),
    InvalidUsername,
    InvalidToken,
}

#[server(GetCountersByUserName, "/api")]
pub async fn get_counters_by_user_name(
    session: UserSession,
) -> Result<CounterResponse, ServerFnError> {
    let pool = extract_pool().await?;

    let user = match backend::auth::get_user(&pool, &session.username, session.token).await {
        Ok(user) => user,
        Err(backend::BackendError::UserNotFound) => return Ok(CounterResponse::InvalidUsername),
        Err(backend::BackendError::InvalidToken) => return Ok(CounterResponse::InvalidToken),
        Err(err) => Err(err)?,
    };

    let data = user.get_counters(&pool).await?;

    let mut counters = Vec::new();
    for db_counter in data {
        counters.push(SerCounter::from_db(&session, db_counter).await?)
    }

    Ok(CounterResponse::Counters(counters))
}

#[server(GetCounterById, "/api")]
pub async fn get_counter_by_id(
    session: UserSession,
    counter_id: uuid::Uuid,
) -> Result<SerCounter, ServerFnError> {
    let pool = extract_pool().await?;
    let _ = backend::auth::get_user(&pool, &session.username, session.token).await?;
    let data =
        backend::get_counter_by_id(&pool, &session.username, session.token, counter_id).await?;

    Ok(SerCounter::from_db(&session, data).await?)
}

#[server(GetPhaseById, "/api")]
pub async fn get_phase_by_id(
    session: UserSession,
    phase_id: uuid::Uuid,
) -> Result<Phase, ServerFnError> {
    let pool = extract_pool().await?;
    let data = backend::get_phase_by_id(&pool, &session.username, session.token, phase_id).await?;
    Ok(data.into())
}

#[server(GetCountableById, "/api")]
pub async fn get_countable_by_id(
    session: UserSession,
    id: uuid::Uuid,
) -> Result<Countable, ServerFnError> {
    let pool = extract_pool().await?;

    if let Ok(counter) =
        backend::get_counter_by_id(&pool, &session.username, session.token, id).await
    {
        return Ok(Countable::Counter(
            SerCounter::from_db(&session, counter.into()).await?,
        ));
    } else if let Ok(phase) =
        backend::get_phase_by_id(&pool, &session.username, session.token, id).await
    {
        return Ok(Countable::Phase(phase.into()));
    } else {
        return Err(backend::BackendError::DataNotFound(String::from(
            "countable",
        )))?;
    }
}

#[server(UpdateCountable, "/api")]
pub async fn update_countable(
    session: UserSession,
    countable: Countable,
) -> Result<(), ServerFnError> {
    match countable {
        Countable::Counter(c) => update_counter(session, c).await?,
        Countable::Phase(p) => update_phase(session, p).await?,
    };

    return Ok(());
}

#[server(CreateCounter, "/api")]
pub async fn create_counter(
    session: UserSession,
    counter: SerCounter,
) -> Result<(), ServerFnError> {
    let pool = extract_pool().await?;

    let _ = backend::update_counter(
        &pool,
        &session.username,
        session.token,
        counter.to_db().await,
    )
    .await?;

    Ok(())
}

#[server(UpdateCounter, "/api")]
pub async fn update_counter(
    session: UserSession,
    counter: SerCounter,
) -> Result<(), ServerFnError> {
    let pool = extract_pool().await?;

    let _ = backend::update_counter(
        &pool,
        &session.username,
        session.token,
        counter.to_db().await,
    )
    .await?;

    futures::future::try_join_all(
        counter
            .phase_list
            .into_iter()
            .map(|countable| update_phase(session.clone(), countable))
            .collect::<Vec<_>>(),
    )
    .await?;

    Ok(())
}

#[server(RemoveCountable, "/api")]
pub async fn remove_countable(session: UserSession, key: uuid::Uuid) -> Result<(), ServerFnError> {
    let countable = get_countable_by_id(session.clone(), key).await?;
    match countable {
        Countable::Counter(c) => remove_counter(session, c.uuid).await?,
        Countable::Phase(p) => remove_phase(session, p.uuid).await?,
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

#[server(CreatePhase, "/api")]
pub async fn create_phase(session: UserSession, phase: Phase) -> Result<(), ServerFnError> {
    let pool = extract_pool().await?;

    backend::update_phase(&pool, &session.username, session.token, phase.to_db()).await?;

    Ok(())
}

#[server(SavePhase, "/api")]
pub async fn update_phase(session: UserSession, phase: Phase) -> Result<(), ServerFnError> {
    let pool = extract_pool().await?;

    backend::update_phase(&pool, &session.username, session.token, phase.to_db()).await?;

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
            save_preferences(session, new_prefs.clone()).await?;
            new_prefs
        }
        Err(err) => return Err(err)?,
    };

    Ok(Preferences::from(prefs))
}

#[server(SavePreferences, "/api")]
pub async fn save_preferences(
    session: UserSession,
    preferences: Preferences,
) -> Result<(), ServerFnError> {
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
    };
    db_prefs
        .db_set(&pool, &session.username, session.token)
        .await?;

    Ok(())
}

#[server(SaveMultiple, "/api")]
pub async fn save_multiple(
    session: UserSession,
    counters: Option<Vec<SerCounter>>,
    phases: Option<Vec<Phase>>,
) -> Result<(), ServerFnError> {
    let pool = extract_pool().await?;

    let counters = counters.unwrap_or_default();
    let phases = phases.unwrap_or_default();

    let save_counter_futures = counters
        .iter()
        .map(|c| update_counter(session.clone(), c.clone()))
        .collect::<Vec<_>>();
    let mut save_phase_futures = counters
        .into_iter()
        .map(|c| {
            c.phase_list
                .into_iter()
                .map(|p| update_phase(session.clone(), p))
                .collect::<Vec<_>>()
        })
        .flatten()
        .collect::<Vec<_>>();

    save_phase_futures.append(
        &mut phases
            .iter()
            .map(|p| update_phase(session.clone(), p.clone()))
            .collect::<Vec<_>>(),
    );

    futures::future::try_join_all(save_counter_futures).await?;
    futures::future::try_join_all(save_phase_futures).await?;

    for phase in phases {
        backend::update_phase(&pool, &session.username, session.token, phase.to_db()).await?;
    }

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

    api::login_user(user.username, password).await?;
    leptos_actix::redirect("/preferences");

    return Ok(session_user);
}
