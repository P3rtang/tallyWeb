use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

mod types;
pub use types::*;

pub async fn create_pool() -> Result<PgPool, sqlx::error::Error> {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url).await?;

    return Ok(pool)
}

pub async fn recreate_db() -> Result<(), sqlx::error::Error> {
    let pool = create_pool().await?;
    execute_multi_file(&pool, "../../postgres/00-recreate-db.sql");
    execute_multi_file(&pool, "../../postgres/01-create-schema.sql");
    return Ok(())
}

pub async fn execute_multi_file(pool: &PgPool, path: &str) -> Result<(), sqlx::error::Error> {
    let file = include_str!("../../postgres/00-recreate-db.sql");
    for line in file.split(";") {
        sqlx::query(line)
            .execute(pool)
            .await?;
    }
    return Ok(())
}

pub async fn get_counter_by_user_id(pool: &PgPool, user_id: i32) -> Result<Vec<DbCounter>, sqlx::error::Error> {
    let counters = sqlx::query_as!(
            DbCounter,
            r#"SELECT * FROM counters WHERE user_id = $1"#,
            user_id
        )
        .fetch_all(pool)
        .await?;

    return Ok(counters)
}

pub async fn get_phase_by_id(pool: &PgPool, phase_id: i32) -> Result<DbPhase, sqlx::error::Error> {
    let phase = sqlx::query_as!(
            DbPhase,
            r#"SELECT * FROM phases WHERE id = $1"#,
            phase_id
        )
        .fetch_one(pool)
        .await?;

    return Ok(phase)
}
