use super::*;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FormData {
    new_username: String,
    password: String,
}

#[server(Update, "/api/session_v2/account")]
pub async fn update(session: UserSession, data: FormData) -> Result<UserSession, ServerFnError> {
    let pool = extract_pool().await?;
    let mut tx = pool.begin().await?;

    let user = backend::auth::change_username(
        &mut tx,
        &session.username,
        &data.new_username,
        &data.password,
    )
    .await?;

    if let Some(token) = user.token {
        tx.commit().await?;
        Ok(UserSession {
            user_uuid: user.uuid,
            username: user.username,
            token,
        })
    } else {
        redirect("/login");
        return Err(BackendError::MissingToken)?;
    }
}
