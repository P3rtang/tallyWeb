#![allow(unused)]
use super::*;

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

const HEX_DIGITS: &str = "0123456789abcdef";

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountAccentColor(pub String);

impl AccountAccentColor {
    fn new(user: &UserSession) -> Self {
        let mut this = Self(String::new());
        this.set_user(user);
        this
    }

    pub fn set_user(&mut self, _user: &UserSession) {
        self.0 = "#8BE9FD".to_string();
    }
}

impl std::fmt::Display for AccountAccentColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<&str> for AccountAccentColor {
    type Error = AppError;

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        if (value.len() == 7 || value.len() == 4)
            && value.starts_with("#")
            && value[1..]
                .chars()
                .all(|c| HEX_DIGITS.contains(c.to_ascii_lowercase()))
        {
            Ok(AccountAccentColor(value.to_string()))
        } else {
            Err(AppError::InvalidColor(value.to_string()))
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct Preferences {
    pub use_default_accent_color: bool,
    pub accent_color: AccountAccentColor,
    pub show_body_border: bool,

    pub show_separator: bool,
    pub save_on_pause: bool,
}

impl Preferences {
    pub fn new(user: &UserSession) -> Self {
        let accent_color = AccountAccentColor::new(user);
        Self {
            use_default_accent_color: true,
            accent_color,
            show_separator: false,
            save_on_pause: true,
            show_body_border: true,
        }
    }
}

#[cfg(feature = "ssr")]
impl Preferences {
    pub fn from_db(user: &UserSession, value: backend::DbPreferences) -> Self {
        Self {
            use_default_accent_color: value.use_default_accent_color,
            accent_color: value
                .accent_color
                .map(|c| AccountAccentColor(c))
                .unwrap_or(AccountAccentColor::new(user)),
            show_separator: value.show_separator,
            save_on_pause: value.save_on_pause,
            show_body_border: value.show_body_border,
        }
    }
}

pub fn provide_prefs(session: RwSignal<UserSession>) -> Resource<Preferences> {
    let prefs_resource = Resource::new_blocking(
        move || session.get(),
        move |user| async move { api::get_user_preferences(user).await.unwrap_or_default() },
    );

    provide_context(prefs_resource);

    prefs_resource
}
