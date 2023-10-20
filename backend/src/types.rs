use chrono::{Duration, Utc};
use rand::Rng;
use sqlx::{query, query_as, PgPool};

use crate::{AuthorizationError, DataError};

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

#[derive(Debug, sqlx::FromRow)]
pub struct DbCounter {
    pub id: i32,
    pub user_id: i32,
    pub name: String,
    pub phases: Vec<i32>,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Debug, sqlx::FromRow)]
pub struct DbPhase {
    pub id: i32,
    pub user_id: i32,
    pub name: String,
    pub count: i32,
    pub time: i64,
    pub hunt_type: Hunttype,
    pub has_charm: bool,
    pub dexnav_encounters: Option<i32>,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Debug)]
pub struct DbUser {
    pub id: i32,
    pub username: String,
    pub password: String,
    pub token: Option<String>,
    pub email: Option<String>,
}

impl DbUser {
    pub async fn new_token(
        &mut self,
        pool: &PgPool,
        dur: Option<Duration>,
    ) -> Result<DbAuthToken, sqlx::error::Error> {
        let mut rng = rand::thread_rng();
        let token_id: u128 = rng.gen();

        query!(
            r#"
            delete from auth_tokens
            where user_id = $1
            "#,
            self.id
        )
        .execute(pool)
        .await?;

        let token = query_as!(
            DbAuthToken,
            r#"
            insert into auth_tokens (id, user_id, expire_on)
            values ($1, $2, $3)

            returning *
            "#,
            format!("{:X}", token_id),
            self.id,
            (Utc::now() + dur.unwrap_or(Duration::days(1))).naive_utc()
        )
        .fetch_one(pool)
        .await?;

        query!(
            r#"
            update users
            set token = $1
            where username = $2 AND password = $3
            "#,
            token.id,
            self.username,
            self.password,
        )
        .execute(pool)
        .await?;

        self.token = Some(token.id.clone());

        Ok(token)
    }
    pub async fn get_token(&self, pool: &PgPool) -> Result<DbAuthToken, sqlx::error::Error> {
        let token = query_as!(
            DbAuthToken,
            r#"
            select * from auth_tokens
            where id = $1 AND user_id = $2
            "#,
            self.token,
            self.id,
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
            where user_id = $1
            "#,
            self.id
        )
        .fetch_all(pool)
        .await?;

        Ok(data)
    }
}

pub struct DbAuthToken {
    pub id: String,
    pub user_id: i32,
    pub expire_on: chrono::NaiveDateTime,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenStatus {
    Valid,
    Invalid,
    Expired,
}

pub struct DbPreferences {
    pub user_id: i32,
    pub use_default_accent_color: bool,
    pub accent_color: Option<String>,
    pub show_separator: bool,
    pub multi_select: bool,
}

impl DbPreferences {
    pub async fn db_get(pool: &PgPool, user_id: i32) -> Result<Self, DataError> {
        let data = match query_as!(
            Self,
            r#"
            select * from preferences
            where user_id = $1
            "#,
            user_id,
        )
        .fetch_one(pool)
        .await
        {
            Ok(data) => data,
            Err(sqlx::Error::RowNotFound) => return Err(DataError::Uninitialized),
            Err(err) => return Err(DataError::Internal(err)),
        };

        Ok(data)
    }

    pub async fn db_set(self, pool: &PgPool, user_id: i32) -> Result<(), sqlx::error::Error> {
        query!(
            r#"
            insert into preferences (user_id, use_default_accent_color, accent_color, show_separator, multi_select)
            values ($1, $2, $3, $4, $5)
            on conflict (user_id)
            do update set use_default_accent_color = $2, accent_color = $3, show_separator = $4, multi_select = $5
            "#,
            user_id,
            self.use_default_accent_color,
            self.accent_color,
            self.show_separator,
            self.multi_select,
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}
