use super::*;
use sqlx::*;

pub async fn get_phases_from_parent_uuid(pool: &PgPool, owner: uuid::Uuid, parent: uuid::Uuid) -> Result<Vec<DbPhase>, BackendError> {
    let phases = query_as(
        r#"
        SELECT * FROM phases
        WHERE owner_uuid = $1 AND parent_uuid = $2
        "#,
    )
    .bind(owner)
    .bind(parent)
    .fetch_all(pool)
    .await?;

    return Ok(phases);
}
