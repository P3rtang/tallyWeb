use super::*;

pub async fn all_by_user(tx: &mut PgTx, user: uuid::Uuid) -> Result<Vec<DbPhase>, BackendError> {
    let phases = sqlx::query_as!(
        DbPhase,
        r#"
        SELECT 
            uuid,
            owner_uuid,
            parent_uuid,
            name,
            count,
            time,
            has_charm,
            hunt_type as "hunt_type: Hunttype",
            dexnav_encounters,
            success,
            created_at
            FROM phases
        where owner_uuid = $1;
        "#,
        user,
    )
    .fetch_all(&mut **tx)
    .await?;

    Ok(phases)
}

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

pub async fn update(tx: &mut PgTx, phase: DbPhase) -> Result<(), BackendError> {
    sqlx::query!(
        r#"
        INSERT INTO phases (uuid, owner_uuid, parent_uuid, name, count, time, hunt_type, has_charm, success, dexnav_encounters, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        ON CONFLICT (uuid) DO UPDATE
        SET
            name = $4,
            count = $5,
            time = $6,
            hunt_type = $7,
            has_charm = $8,
            success = $9,
            dexnav_encounters = $10
        "#,
        phase.uuid,
        phase.owner_uuid,
        phase.parent_uuid,
        phase.name,
        phase.count,
        phase.time,
        phase.hunt_type as Hunttype,
        phase.has_charm,
        phase.success,
        phase.dexnav_encounters,
        phase.created_at,
    )
    .execute(&mut **tx)
    .await?;

    Ok(())
}
