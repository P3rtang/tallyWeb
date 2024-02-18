use std::error::Error;
use thiserror::Error;

use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
pub use sqlx::PgPool;

pub mod auth;
mod types;
pub use types::*;
mod counter_data;
pub use counter_data::*;

#[derive(Debug, thiserror::Error)]
pub enum BackendError {
    #[error("Invalid Token")]
    InvalidToken,
    #[error("Database Error: {0}")]
    DatabaseError(#[from] sqlx::error::Error),
    #[error("Counter data not found")]
    CounterNotFound,
    #[error("Not authorized to access data")]
    Unauthorized,
    #[error("User data not found")]
    UserNotFound,
    #[error("Internal Server Error\nGot Error: {0}")]
    Internal(String),
    #[error("Username already exists")]
    UserExists,
    #[error("Invalid Username or Password")]
    InvalidSecrets,
    #[error("Invalid Password provided")]
    InvalidPassword,
    #[error("Invalid Username provided")]
    InvalidUsername,
    #[error("Could not find {0} data for user")]
    DataNotFound(String),
}

pub trait DatabaseError: Error {}

#[derive(Debug, thiserror::Error)]
pub enum SignupError {
    #[error("Username already exists")]
    UserExists,
    #[error("Internal Server error when creating user\nGot Error: {0}")]
    Internal(String),
    #[error("Failed to generate token for new user")]
    GenerateToken,
}

impl DatabaseError for SignupError {}

#[derive(Debug, thiserror::Error)]
pub enum ChangeUserError {
    #[error("Username already exists")]
    UserExists,
    #[error("User provided the wrong password")]
    InvalidPassword(#[from] AuthorizationError),
    #[error("Internal Server error when modifying user")]
    Internal(#[from] sqlx::Error),
}

impl DatabaseError for ChangeUserError {}

#[derive(Debug, Error)]
pub enum LoginError {
    #[error("Account does not exist")]
    InvalidUsername,
    #[error("User provided the wrong password")]
    InvalidPassword,
    #[error("Provided username or password was incorrect")]
    InvalidSecrets,
    #[error("Internal Server error when logging in user\nGot Error: {0}")]
    Internal(String),
}

impl DatabaseError for LoginError {}

#[derive(Debug, Error)]
pub enum AuthorizationError {
    #[error("Provided Token is expired")]
    ExpiredToken,
    #[error("Provided Token is Invalid")]
    InvalidToken,
    #[error("Provided Username does not exist")]
    UserNotFound,
    #[error("Provided Password is incorrect")]
    InvalidPassword,
    #[error("Internal Server error when checking AuthToken\nGot Error: {0}")]
    Internal(String),
    #[error("Provide Username is Invalid")]
    InvalidUsername,
}

impl DatabaseError for AuthorizationError {}

pub async fn create_pool() -> Result<PgPool, sqlx::error::Error> {
    dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;

    Ok(pool)
}

pub async fn recreate_db() -> Result<(), sqlx::error::Error> {
    let pool = create_pool().await?;
    execute_multi_file(&pool, "../../postgres/00-recreate-db.sql").await?;
    execute_multi_file(&pool, "../../postgres/01-create-schema.sql").await?;
    Ok(())
}

pub async fn execute_multi_file(pool: &PgPool, _path: &str) -> Result<(), sqlx::error::Error> {
    let file = include_str!("../../postgres/00-recreate-db.sql");
    for line in file.split(';') {
        sqlx::query(line).execute(pool).await?;
    }
    Ok(())
}

pub async fn get_counter_by_id(
    pool: &PgPool,
    username: &str,
    token: uuid::Uuid,
    uuid: uuid::Uuid,
) -> Result<DbCounter, BackendError> {
    let user = auth::get_user(pool, username, token).await?;

    let counter = match sqlx::query_as!(
        DbCounter,
        r#"
        SELECT * FROM counters
        WHERE uuid = $1 AND owner_uuid = $2
        "#,
        uuid,
        user.uuid,
    )
    .fetch_one(pool)
    .await
    {
        Ok(counter) => counter,
        Err(sqlx::Error::RowNotFound) => Err(BackendError::CounterNotFound)?,
        Err(err) => Err(err)?,
    };

    if counter.owner_uuid != user.uuid {
        Err(BackendError::Unauthorized)?;
    }

    Ok(counter)
}

pub async fn get_phase_by_id(
    pool: &PgPool,
    username: &str,
    token: uuid::Uuid,
    phase_id: uuid::Uuid,
) -> Result<DbPhase, BackendError> {
    let user = auth::get_user(pool, username, token).await?;

    let phase: DbPhase = sqlx::query_as("SELECT * FROM phases WHERE uuid = $1 AND owner_uuid = $2")
        .bind(phase_id)
        .bind(user.uuid)
        .fetch_one(pool)
        .await?;

    Ok(phase)
}

pub async fn update_phase(
    pool: &PgPool,
    username: &str,
    token: uuid::Uuid,
    phase: DbPhase,
) -> Result<(), BackendError> {
    let _ = auth::get_user(pool, username, token).await?;
    let _ = sqlx::query(
        r#"
        INSERT INTO phases (uuid, owner_uuid, parent_uuid, name, count, time, hunt_type, has_charm, success)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)

        ON CONFLICT (uuid) DO UPDATE
            SET name      = $4,
                count     = $5,
                time      = $6,
                hunt_type = $7,
                has_charm = $8,
                success   = $9
        "#,
    )
    .bind(phase.uuid)
    .bind(phase.owner_uuid)
    .bind(phase.parent_uuid)
    .bind(phase.name)
    .bind(phase.count)
    .bind(phase.time)
    .bind(phase.hunt_type)
    .bind(phase.has_charm)
    .bind(phase.success)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn update_counter(
    pool: &PgPool,
    username: &str,
    token: uuid::Uuid,
    counter: DbCounter,
) -> Result<(), BackendError> {
    let _ = auth::get_user(pool, username, token).await?;
    sqlx::query!(
        r#"
        INSERT INTO counters (uuid, owner_uuid, name)
        VALUES ($1, $2, $3)

        ON CONFLICT (uuid) DO UPDATE
            SET name = $3
        "#,
        counter.uuid,
        counter.owner_uuid,
        counter.name,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn remove_counter(
    pool: &PgPool,
    username: &str,
    token: uuid::Uuid,
    counter_uuid: uuid::Uuid,
) -> Result<(), BackendError> {
    let user = auth::get_user(pool, username, token).await?;

    sqlx::query!(
        r#"
        delete from counters
        where owner_uuid = $1 AND uuid = $2
        "#,
        user.uuid,
        counter_uuid,
    )
    .execute(pool)
    .await?;

    sqlx::query!(
        r#"
        DELETE FROM phases
        WHERE owner_uuid = $1 AND parent_uuid = $2
        "#,
        user.uuid,
        counter_uuid,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn remove_phase(pool: &PgPool, phase_id: uuid::Uuid) -> Result<(), sqlx::error::Error> {
    sqlx::query!(
        r#"
        delete from phases
        where uuid = $1
        "#,
        phase_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn migrate(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::migrate!("../migrations").run(pool).await?;

    Ok(())
}
