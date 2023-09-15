use std::error::Error;
use thiserror::Error;

use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

pub mod auth;
mod types;
pub use types::*;

pub trait DatabaseError: Error {}

#[derive(Debug, thiserror::Error)]
pub enum SignupError {
    #[error("Could not create new User, username already exists")]
    UserExists,
    #[error("Internal Server error when creating user")]
    Internal(#[from] sqlx::Error),
}

impl DatabaseError for SignupError {}

#[derive(Debug, Error)]
pub enum LoginError {
    #[error("Account does not exist")]
    InvalidUsername,
    #[error("User provided the wrong password")]
    InvalidPassword,
    #[error("Internal Server error when logging in user")]
    Internal(#[from] sqlx::Error),
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
    #[error("Internal Server error when checking AuthToken")]
    Internal(#[from] sqlx::Error),
}

impl DatabaseError for AuthorizationError {}

pub async fn create_pool() -> Result<PgPool, sqlx::error::Error> {
    dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;

    return Ok(pool);
}

pub async fn recreate_db() -> Result<(), sqlx::error::Error> {
    let pool = create_pool().await?;
    execute_multi_file(&pool, "../../postgres/00-recreate-db.sql").await?;
    execute_multi_file(&pool, "../../postgres/01-create-schema.sql").await?;
    return Ok(());
}

pub async fn execute_multi_file(pool: &PgPool, _path: &str) -> Result<(), sqlx::error::Error> {
    let file = include_str!("../../postgres/00-recreate-db.sql");
    for line in file.split(";") {
        sqlx::query(line).execute(pool).await?;
    }
    return Ok(());
}

pub async fn get_counter_by_id(pool: &PgPool, id: i32) -> Result<DbCounter, sqlx::error::Error> {
    let counter = sqlx::query_as!(DbCounter, r#"SELECT * FROM counters WHERE id = $1"#, id,)
        .fetch_one(pool)
        .await?;

    return Ok(counter);
}

pub async fn get_phase_by_id(pool: &PgPool, phase_id: i32) -> Result<DbPhase, sqlx::error::Error> {
    let phase = sqlx::query_as!(DbPhase, r#"SELECT * FROM phases WHERE id = $1"#, phase_id)
        .fetch_one(pool)
        .await?;

    return Ok(phase);
}

pub async fn create_counter(
    pool: &PgPool,
    user_id: i32,
    name: String,
) -> Result<i32, sqlx::error::Error> {
    struct Record {
        id: i32,
    }
    let record = sqlx::query_as!(
        Record,
        r#"
            INSERT INTO counters (user_id, name)
            VALUES ($1, $2)
            RETURNING id
            "#,
        user_id,
        name,
    )
    .fetch_one(pool)
    .await?;

    return Ok(record.id);
}

pub async fn create_phase(pool: &PgPool, name: String) -> Result<i32, sqlx::error::Error> {
    struct Record {
        id: i32,
    }
    let record = sqlx::query_as!(
        Record,
        r#"
            INSERT INTO phases (name, count, time)
            VALUES ($1, $2, $3)
            RETURNING id
            "#,
        name,
        0,
        0,
    )
    .fetch_one(pool)
    .await?;
    return Ok(record.id);
}

pub async fn assign_phase(
    pool: &PgPool,
    counter_id: i32,
    phase_id: i32,
) -> Result<(), sqlx::error::Error> {
    let counter = get_counter_by_id(pool, counter_id).await?;
    let mut phases = counter.phases;
    phases.push(phase_id);

    let _ = sqlx::query!(
        r#"
            UPDATE counters
            SET phases = $2
            WHERE id = $1
            "#,
        counter_id,
        &phases,
    )
    .execute(pool)
    .await?;
    return Ok(());
}

pub async fn update_phase(pool: &PgPool, phase: DbPhase) -> Result<(), sqlx::error::Error> {
    let _ = sqlx::query!(
        r#"
            UPDATE phases
            SET name = $2, count = $3, time = $4
            WHERE id = $1
            "#,
        phase.id,
        phase.name,
        phase.count,
        phase.time,
    )
    .execute(pool)
    .await?;

    return Ok(());
}

pub async fn update_counter(pool: &PgPool, counter: DbCounter) -> Result<(), sqlx::error::Error> {
    sqlx::query!(
        r#"
        UPDATE counters
        SET name = $2, phases = $3
        WHERE id = $1
        "#,
        counter.id,
        counter.name,
        &counter.phases,
    )
    .execute(pool)
    .await?;

    return Ok(());
}

pub async fn remove_counter(
    pool: &PgPool,
    user_id: i32,
    counter_id: i32,
) -> Result<(), sqlx::error::Error> {
    sqlx::query!(
        r#"
        delete from counters
        where user_id = $1 AND id = $2
        "#,
        user_id,
        counter_id,
    )
    .execute(pool)
    .await?;

    return Ok(());
}

pub async fn remove_phase(pool: &PgPool, phase_id: i32) -> Result<(), sqlx::error::Error> {
    sqlx::query!(
        r#"
        UPDATE counters
        SET phases = array_remove(phases, $1)
        WHERE $1 = ANY(phases);
        "#,
        phase_id,
    )
    .execute(pool)
    .await?;

    sqlx::query!(
        r#"
        delete from phases
        where id = $1
        "#,
        phase_id,
    )
    .execute(pool)
    .await?;

    return Ok(());
}
