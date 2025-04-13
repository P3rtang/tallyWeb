#![allow(unused)]

use super::*;

impl Screen {
    pub fn set_cookie(&self) -> AppResult<()> {
        let document = document();
        let document: &web_sys::HtmlDocument = document.unchecked_ref();

        let size_json = serde_json::to_string(self)?;

        let size_cookie = cookie::Cookie::build(("screen_size", &size_json))
            .path("/")
            .same_site(cookie::SameSite::Strict)
            .build();
        let size_cookie_encoded = cookie::Cookie::encoded(&size_cookie);

        let mut cookie_str = size_cookie_encoded.to_string();
        let age_str = format!("; Max-Age={}", 30 * 24 * 60 * 60);
        cookie_str.push_str(&age_str);

        document.set_cookie(&cookie_str).ok();

        Ok(())
    }
}
