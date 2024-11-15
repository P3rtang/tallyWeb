use super::*;

pub async fn all_by_user(tx: &mut PgTx, user: uuid::Uuid) -> Result<Vec<DbCounter>, BackendError> {
    let counters = sqlx::query_as!(
        DbCounter,
        r#"
        SELECT * FROM counters
        where owner_uuid = $1;
        "#,
        user,
    )
    .fetch_all(&mut **tx)
    .await?;

    Ok(counters)
}

pub async fn get_children(tx: &mut PgTx, key: uuid::Uuid) -> Result<Vec<DbPhase>, BackendError> {
    let last_child = sqlx::query_as!(
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
            last_edit,
            created_at,
            is_deleted,
            step_size
            FROM phases
        WHERE parent_uuid = $1
        ORDER BY created_at;
        "#,
        key
    )
    .fetch_all(&mut **tx)
    .await?;

    Ok(last_child)
}

pub async fn edited(tx: &mut PgTx, key: uuid::Uuid) -> Result<(), BackendError> {
    sqlx::query!(
        r#"
        UPDATE counters
        SET last_edit = $2
        WHERE uuid = $1
        "#,
        key,
        chrono::Utc::now().naive_utc(),
    )
    .execute(&mut **tx)
    .await?;

    Ok(())
}

pub async fn set_name(tx: &mut PgTx, key: uuid::Uuid, name: &str) -> Result<(), BackendError> {
    sqlx::query!(
        r#"
        UPDATE counters
        SET name = $2
        WHERE uuid = $1
        "#,
        key,
        name,
    )
    .execute(&mut **tx)
    .await?;

    edited(tx, key).await?;

    Ok(())
}

pub async fn get_count(tx: &mut PgTx, key: uuid::Uuid) -> Result<i32, BackendError> {
    struct Count {
        count: Option<i64>,
    }
    let count = sqlx::query_as!(
        Count,
        r#"
        SELECT SUM(count) AS count FROM phases
        WHERE parent_uuid = $1
        "#,
        key
    )
    .fetch_one(&mut **tx)
    .await?;

    Ok(count.count.unwrap_or_default() as i32)
}

pub async fn set_count(tx: &mut PgTx, key: uuid::Uuid, count: i32) -> Result<(), BackendError> {
    let children = get_children(tx, key).await?;
    if children.is_empty() {
        return Ok(());
    }

    let cur_count = get_count(tx, key).await?;
    let mut diff = count - cur_count;

    for i in (0..children.len()).rev() {
        if diff + children[i].count <= 0 {
            phase::set_count(tx, children[i].uuid, 0).await?;
            diff += children[i].count
        } else {
            phase::set_count(tx, children[i].uuid, children[i].count + diff).await?;
            break;
        }
    }

    Ok(())
}

pub async fn set_step(tx: &mut PgTx, key: uuid::Uuid, step: i32) -> Result<(), BackendError> {
    let children = get_children(tx, key).await?;

    if let Some(child) = children.last() {
        phase::set_step(tx, child.uuid, step).await?;
    }

    Ok(())
}

pub async fn get_time(tx: &mut PgTx, key: uuid::Uuid) -> Result<i64, BackendError> {
    struct Time {
        time: Option<i64>,
    }
    let time = sqlx::query_as!(
        Time,
        r#"
        SELECT CAST(SUM(time::numeric) AS bigint) AS time FROM phases
        WHERE parent_uuid = $1
        "#,
        key
    )
    .fetch_one(&mut **tx)
    .await?;

    Ok(time.time.unwrap_or_default())
}

pub async fn set_time(tx: &mut PgTx, key: uuid::Uuid, time: i64) -> Result<(), BackendError> {
    let children = get_children(tx, key).await?;
    if children.is_empty() {
        return Ok(());
    }

    let mut diff = time - get_time(tx, key).await?;

    for i in (0..children.len()).rev() {
        if diff + children[i].time <= 0 {
            phase::set_time(tx, children[i].uuid, 0).await?;
            diff += children[i].time
        } else {
            phase::set_time(tx, children[i].uuid, children[i].time + diff).await?;
            break;
        }
    }

    Ok(())
}

pub async fn set_hunttype(
    tx: &mut PgTx,
    key: uuid::Uuid,
    hunttype: Hunttype,
) -> Result<(), BackendError> {
    sqlx::query_unchecked!(
        r#"
        UPDATE phases
        SET 
            hunt_type = $2,
            last_edit = $3
        WHERE parent_uuid = $1
        "#,
        key,
        hunttype,
        chrono::Utc::now().naive_utc(),
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
        SET 
            has_charm = $2,
            last_edit = $3
        WHERE parent_uuid = $1
        "#,
        key,
        has_charm,
        chrono::Utc::now().naive_utc(),
    )
    .execute(&mut **tx)
    .await?;

    Ok(())
}

pub async fn update(tx: &mut PgTx, counter: DbCounter) -> Result<(), BackendError> {
    sqlx::query!(
        r#"
        INSERT INTO counters (uuid, owner_uuid, name, created_at, is_deleted)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (uuid) DO UPDATE
        SET
            name = $3,
            is_deleted = $5
        "#,
        counter.uuid,
        counter.owner_uuid,
        counter.name,
        counter.created_at,
        counter.is_deleted,
    )
    .execute(&mut **tx)
    .await?;

    edited(tx, counter.uuid).await?;

    Ok(())
}

pub async fn archive(tx: &mut PgTx, key: uuid::Uuid) -> Result<(), BackendError> {
    sqlx::query!(
        r#"
        UPDATE counters
        SET is_deleted = true
        WHERE uuid = $1
        "#,
        key,
    )
    .execute(&mut **tx)
    .await?;

    sqlx::query!(
        r#"
        UPDATE phases
        SET
            is_deleted = true,
            last_edit = $2
        WHERE parent_uuid = $1
        "#,
        key,
        chrono::Utc::now().naive_utc(),
    )
    .execute(&mut **tx)
    .await?;

    edited(tx, key).await?;

    Ok(())
}
