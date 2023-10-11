use pbkdf2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Pbkdf2,
};
use sqlx::{query_as, PgPool};

use crate::{AuthorizationError, DatabaseError, DbUser, LoginError, SignupError, TokenStatus};

pub async fn insert_user(
    pool: &PgPool,
    username: String,
    password: String,
) -> Result<DbUser, SignupError> {
    // generate salt
    let salt = SaltString::generate(&mut OsRng);
    let mut params = pbkdf2::Params::default();
    params.rounds = 100_000;

    // Hash password to PHC string ($pbkdf2-sha256$...
    let hashed_password = Pbkdf2
        .hash_password_customized(password.as_bytes(), None, None, params, &salt)
        // .hash_password(password.as_bytes(), &salt)
        .unwrap()
        .to_string();

    let mut user = match query_as!(
        DbUser,
        r#"
        insert into users (username, password)
        values ($1, $2)
        returning *
        "#,
        username,
        hashed_password,
    )
    .fetch_one(pool)
    .await
    {
        Ok(user) => user,
        Err(sqlx::Error::Database(error)) if error.constraint() == Some("users_username_key") => {
            return Err(SignupError::UserExists);
        }
        Err(err) => return Err(SignupError::Internal(err)),
    };

    user.new_token(pool, None).await?;

    return Ok(user);
}

pub async fn login_user(
    pool: &PgPool,
    username: String,
    password: String,
    remember: bool,
) -> Result<DbUser, impl DatabaseError> {
    let mut user = match query_as!(
        DbUser,
        r#"
        select * from users
        where username = $1
        "#,
        username,
    )
    .fetch_one(pool)
    .await
    {
        Ok(user) => user,
        Err(_) => return Err(LoginError::InvalidUsername),
    };

    let parsed_hash = PasswordHash::new(&user.password).unwrap();
    // Trait objects for algorithms to support
    let algs: &[&dyn PasswordVerifier] = &[&Pbkdf2];
    if let Err(_err) = parsed_hash.verify_password(algs, password) {
        return Err(LoginError::InvalidPassword);
    };

    let dur = if remember {
        Some(chrono::Duration::days(30))
    } else {
        None
    };

    user.new_token(pool, dur).await.map_err(|err| {
        println!("{err}");
        err
    })?;

    return Ok(user);
}

pub async fn get_user(
    pool: &PgPool,
    username: String,
    token: String,
) -> Result<DbUser, AuthorizationError> {
    let user = match query_as!(
        DbUser,
        r#"
        select * from users
        where username = $1
        "#,
        username,
    )
    .fetch_one(pool)
    .await
    {
        Ok(user) => user,
        Err(sqlx::Error::RowNotFound) => return Err(AuthorizationError::UserNotFound),
        Err(err) => Err(err)?,
    };

    if user.token != Some(token) {
        return Err(AuthorizationError::InvalidToken);
    }

    match user.token_status(pool).await {
        TokenStatus::Expired => return Err(AuthorizationError::ExpiredToken),
        // TODO: add logging for invalid token use, user might be suspicious
        TokenStatus::Invalid => return Err(AuthorizationError::InvalidToken),
        TokenStatus::Valid => {}
    };

    return Ok(user);
}
