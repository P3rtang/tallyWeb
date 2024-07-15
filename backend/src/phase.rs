use super::*;

pub async fn set_name(tx: &mut PgTx, key: uuid::Uuid, name: &str) -> Result<(), BackendError> {
    sqlx::query!(
        r#"
        UPDATE phases
        SET name = $2
        WHERE uuid = $1
        "#,
        key,
        name,
    )
    .execute(&mut **tx)
    .await?;

    Ok(())
}

pub async fn set_count(tx: &mut PgTx, key: uuid::Uuid, count: i32) -> Result<(), BackendError> {
    sqlx::query!(
        r#"
        UPDATE phases
        SET count = $2
        WHERE uuid = $1
        "#,
        key,
        count,
    )
    .execute(&mut **tx)
    .await?;

    Ok(())
}

pub async fn set_time(tx: &mut PgTx, key: uuid::Uuid, time: i64) -> Result<(), BackendError> {
    sqlx::query!(
        r#"
        UPDATE phases
        SET time = $2
        WHERE uuid = $1
        "#,
        key,
        time,
    )
    .execute(&mut **tx)
    .await?;

    Ok(())
}

pub async fn set_hunttype(
    tx: &mut PgTx,
    key: uuid::Uuid,
    hunttype: Hunttype,
) -> Result<(), BackendError> {
    sqlx::query!(
        r#"
        UPDATE phases
        SET hunt_type = $2
        WHERE uuid = $1
        "#,
        key,
        hunttype as Hunttype,
    )
    .execute(&mut **tx)
    .await?;

    Ok(())
}

pub async fn set_charm(
    tx: &mut PgTx,
    key: uuid::Uuid,
    has_charm: bool,
) -> Result<(), BackendError> {
    sqlx::query!(
        r#"
        UPDATE phases
        SET has_charm = $2
        WHERE uuid = $1
        "#,
        key,
        has_charm,
    )
    .execute(&mut **tx)
    .await?;

    Ok(())
}
