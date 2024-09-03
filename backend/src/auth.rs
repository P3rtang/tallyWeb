use pbkdf2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Pbkdf2,
};
use sqlx::*;

use super::*;

pub async fn insert_user(
    pool: &PgPool,
    username: &str,
    password: &str,
) -> Result<DbUser, BackendError> {
    // generate salt
    let salt = SaltString::generate(&mut OsRng);
    let params = pbkdf2::Params {
        rounds: 100_000,
        ..Default::default()
    };

    // Hash password to PHC string ($pbkdf2-sha256$...
    let hashed_password = Pbkdf2
        .hash_password_customized(password.as_bytes(), None, None, params, &salt)
        // .hash_password(password.as_bytes(), &salt)
        .unwrap()
        .to_string();

    match query!(
        r#"
        insert into users (username, password)
        values ($1, $2)
        "#,
        username,
        hashed_password,
    )
    .execute(pool)
    .await
    {
        Ok(_) => {}
        Err(sqlx::Error::Database(error)) if error.constraint() == Some("users_username_key") => {
            return Err(BackendError::UserExists);
        }
        Err(err) => return Err(err)?,
    };

    let token = new_token(pool, username, password, chrono::Duration::days(1)).await?;
    let user = get_user(pool, username, token.uuid).await?;

    Ok(user)
}

async fn new_token(
    pool: &PgPool,
    username: &str,
    password: &str,
    token_dur: chrono::Duration,
) -> Result<DbAuthToken, BackendError> {
    struct UserId {
        uuid: uuid::Uuid,
    }

    let token_uuid = uuid::Uuid::new_v4();

    check_pass(pool, username, password).await?;

    let id = query_as!(
        UserId,
        r#"
        select uuid from users
        where username = $1
        "#,
        username,
    )
    .fetch_one(pool)
    .await
    .map_err(|_| BackendError::InvalidSecrets)?;

    let token = query_as!(
        DbAuthToken,
        r#"
        insert into auth_tokens (uuid, user_uuid, expire_on)
        values ($1, $2, $3)

        returning *
        "#,
        token_uuid,
        id.uuid,
        chrono::Utc::now().naive_utc().checked_add_signed(token_dur),
    )
    .fetch_one(pool)
    .await?;

    Ok(token)
}

pub async fn login_user(
    pool: &PgPool,
    username: String,
    password: String,
    token_dur: chrono::Duration,
) -> Result<DbUser, BackendError> {
    struct PassUser {
        password: String,
    }

    let pass = match query_as!(
        PassUser,
        r#"
        select users.password from users
        where username = $1
        "#,
        username,
    )
    .fetch_one(pool)
    .await
    {
        Ok(user) => user,
        Err(_) => return Err(BackendError::InvalidUsername),
    };

    let parsed_hash = PasswordHash::new(&pass.password).unwrap();
    // Trait objects for algorithms to support
    let algs: &[&dyn PasswordVerifier] = &[&Pbkdf2];
    if let Err(_err) = parsed_hash.verify_password(algs, &password) {
        return Err(BackendError::InvalidPassword);
    };

    let token = new_token(pool, &username, &password, token_dur).await?;
    let user = get_user(pool, &username, token.uuid).await?;

    Ok(user)
}

pub async fn change_password(
    pool: &PgPool,
    username: String,
    old_pass: String,
    new_pass: String,
) -> Result<(), impl DatabaseError> {
    struct PassUser {
        password: String,
    }

    let user = match query_as!(
        PassUser,
        r#"
        select users.password from users
        where username = $1
        "#,
        username,
    )
    .fetch_one(pool)
    .await
    {
        Ok(user) => user,
        Err(_) => return Err(AuthorizationError::InvalidUsername),
    };

    let parsed_hash = PasswordHash::new(&user.password).unwrap();
    // Trait objects for algorithms to support
    let algs: &[&dyn PasswordVerifier] = &[&Pbkdf2];
    if let Err(_err) = parsed_hash.verify_password(algs, &old_pass) {
        return Err(AuthorizationError::InvalidPassword);
    };

    // generate salt
    let salt = SaltString::generate(&mut OsRng);
    let params = pbkdf2::Params {
        rounds: 100_000,
        ..Default::default()
    };

    // Hash password to PHC string ($pbkdf2-sha256$...
    let hashed_password = Pbkdf2
        .hash_password_customized(new_pass.as_bytes(), None, None, params, &salt)
        // .hash_password(password.as_bytes(), &salt)
        .unwrap()
        .to_string();

    match query!(
        r#"
        update users
        set password=$2
        where username=$1
        "#,
        username,
        hashed_password,
    )
    .fetch_one(pool)
    .await
    {
        Ok(_) => {}
        Err(sqlx::Error::RowNotFound) => return Err(AuthorizationError::InvalidPassword)?,
        Err(err) => Err(AuthorizationError::Internal(err.to_string()))?,
    };

    Ok(())
}

pub async fn change_username(
    pool: &PgPool,
    old_username: &str,
    new_username: &str,
    password: &str,
) -> Result<DbUser, BackendError> {
    check_pass(pool, old_username, password).await?;

    match query!(
        r#"
        update users
        set username=$2
        where username=$1
        "#,
        old_username,
        new_username,
    )
    .execute(pool)
    .await
    {
        Ok(_) => {}
        Err(sqlx::Error::Database(err)) if err.constraint() == Some("users_username_key") => {
            Err(BackendError::UserExists)?
        }
        Err(err) => Err(err)?,
    };

    let user = query_as!(
        DbUser,
        r#"
        select users.uuid, users.username, tokens.uuid as token, tokens.expire_on as token_expire, users.email
        from users join auth_tokens as tokens on tokens.user_uuid = users.uuid
        where users.username = $1
        "#,
        new_username,
    )
    .fetch_one(pool)
    .await?;

    Ok(user)
}

pub async fn check_pass(pool: &PgPool, username: &str, password: &str) -> Result<(), BackendError> {
    struct Pass {
        password: String,
    }
    let pass = query_as!(
        Pass,
        r#"
        select password from users
        where username = $1
        "#,
        username,
    )
    .fetch_one(pool)
    .await
    .map_err(|_| BackendError::InvalidUsername)?;

    let parsed_hash = PasswordHash::new(&pass.password).unwrap();
    // Trait objects for algorithms to support
    let algs: &[&dyn PasswordVerifier] = &[&Pbkdf2];
    if let Err(_err) = parsed_hash.verify_password(algs, password) {
        return Err(BackendError::InvalidPassword);
    };

    Ok(())
}

pub async fn get_user(
    pool: &PgPool,
    username: &str,
    token: uuid::Uuid,
) -> Result<DbUser, BackendError> {
    let user = match query_as!(
        DbUser,
        r#"
        select 
            users.uuid as uuid,
            users.username,
            tokens.uuid as token,
            tokens.expire_on as token_expire,
            users.email
        from users join auth_tokens as tokens on users.uuid = tokens.user_uuid
        where username = $1 and tokens.uuid = $2
        "#,
        username,
        token,
    )
    .fetch_one(pool)
    .await
    {
        Ok(user) => user,
        Err(sqlx::Error::RowNotFound) => return Err(BackendError::UserNotFound),
        Err(err) => Err(err)?,
    };

    Ok(user)
}

pub enum SessionState {
    Valid,
    Expired,
}

pub async fn check_user(
    pool: &PgPool,
    username: &str,
    token: uuid::Uuid,
) -> Result<SessionState, BackendError> {
    let user = get_user(pool, username, token).await?;

    if user.token_expire < chrono::Utc::now().naive_utc() {
        Ok(SessionState::Expired)
    } else {
        Ok(SessionState::Valid)
    }
}
