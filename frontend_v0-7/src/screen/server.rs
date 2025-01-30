use super::*;

/**
 * Gets the screen size from a cookie in the request
 */
#[server]
pub async fn get_screen() -> Result<Screen, ServerFnError> {
    use leptos_actix::extract;

    let header = extract::<actix_web::HttpRequest>().await?;
    let cookie = match header.cookie("screen_size") {
        Some(s) => s.value().to_string(),
        None => return Ok(Screen::default()),
    };

    let screen: Screen = serde_json::from_str(&cookie)?;

    Ok(screen)
}
