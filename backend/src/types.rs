use super::*;
use sqlx::{query, query_as, PgPool};

#[derive(Debug, Clone, sqlx::Type)]
#[sqlx(type_name = "hunttype")]
pub enum Hunttype {
    OldOdds,
    NewOdds,
    SOS,
    DexNav,
    MasudaGenIV,
    MasudaGenV,
    MasudaGenVI,
}

impl From<String> for Hunttype {
    fn from(value: String) -> Self {
        match value.as_str() {
            "OldOdds" => Self::OldOdds,
            "NewOdds" => Self::NewOdds,
            "SOS" => Self::SOS,
            "DexNav" => Self::DexNav,
            "MasudaGenIV" => Self::MasudaGenIV,
            "MasudaGenV" => Self::MasudaGenV,
            "MasudaGenVI" => Self::MasudaGenVI,
            _ => Self::NewOdds,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct DbCounter {
    pub uuid: uuid::Uuid,
    pub owner_uuid: uuid::Uuid,
    pub name: String,
    pub created_at: chrono::NaiveDateTime,
    pub last_edit: chrono::NaiveDateTime,
    pub is_deleted: bool,
}

#[derive(Debug, sqlx::FromRow)]
pub struct DbPhase {
    pub uuid: uuid::Uuid,
    pub owner_uuid: uuid::Uuid,
    pub parent_uuid: uuid::Uuid,
    pub name: String,
    pub count: i32,
    pub time: i64,
    pub hunt_type: Hunttype,
    pub has_charm: bool,
    pub dexnav_encounters: Option<i32>,
    pub success: bool,
    pub created_at: chrono::NaiveDateTime,
    pub last_edit: chrono::NaiveDateTime,
    pub is_deleted: bool,
}

#[derive(Debug)]
pub struct DbUser {
    pub uuid: uuid::Uuid,
    pub username: String,
    pub token: Option<uuid::Uuid>,
    pub email: Option<String>,
}

impl DbUser {
    pub async fn get_token(&self, pool: &PgPool) -> Result<DbAuthToken, sqlx::error::Error> {
        let token = query_as!(
            DbAuthToken,
            r#"
            select * from auth_tokens
            where uuid = $1 AND user_uuid = $2
            "#,
            self.token,
            self.uuid,
        )
        .fetch_one(pool)
        .await?;
        Ok(token)
    }

    pub async fn token_status(&self, pool: &PgPool) -> TokenStatus {
        if let Ok(token) = self.get_token(pool).await {
            if token.expire_on.and_utc() > chrono::Utc::now() {
                TokenStatus::Valid
            } else {
                TokenStatus::Expired
            }
        } else {
            TokenStatus::Invalid
        }
    }

    pub async fn get_counters(&self, pool: &PgPool) -> Result<Vec<DbCounter>, AuthorizationError> {
        let data = query_as!(
            DbCounter,
            r#"
            select * from counters
            where owner_uuid = $1
            "#,
            self.uuid
        )
        .fetch_all(pool)
        .await
        .map_err(|err| AuthorizationError::Internal(err.to_string()))?;

        Ok(data)
    }
}

pub struct DbAuthToken {
    pub uuid: uuid::Uuid,
    pub user_uuid: uuid::Uuid,
    pub expire_on: chrono::NaiveDateTime,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenStatus {
    Valid,
    Invalid,
    Expired,
}

pub struct DbPreferences {
    pub user_uuid: uuid::Uuid,
    pub use_default_accent_color: bool,
    pub accent_color: Option<String>,
    pub show_separator: bool,
    pub multi_select: bool,
    pub save_on_pause: bool,
}

impl DbPreferences {
    pub async fn db_get(pool: &PgPool, user_uuid: uuid::Uuid) -> Result<Self, BackendError> {
        let data = match query_as!(
            DbPreferences,
            r#"
            select * from preferences
            where user_uuid = $1
            "#,
            user_uuid,
        )
        .fetch_one(pool)
        .await
        {
            Ok(data) => data,
            Err(sqlx::Error::RowNotFound) => {
                Err(BackendError::DataNotFound(String::from("preferences")))?
            }
            Err(err) => Err(err)?,
        };

        Ok(data)
    }

    pub async fn db_set(
        self,
        pool: &PgPool,
        username: &str,
        token: uuid::Uuid,
    ) -> Result<(), BackendError> {
        let user = auth::get_user(pool, username, token).await?;
        query!(
            r#"
            INSERT INTO preferences (
                user_uuid,
                use_default_accent_color,
                accent_color,
                show_separator,
                multi_select,
                save_on_pause
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (user_uuid) DO UPDATE
                SET use_default_accent_color = $2,
                    accent_color = $3,
                    show_separator = $4,
                    multi_select = $5,
                    save_on_pause = $6
            "#,
            user.uuid,
            self.use_default_accent_color,
            self.accent_color,
            self.show_separator,
            self.multi_select,
            self.save_on_pause,
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}
