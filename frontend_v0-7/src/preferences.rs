#![allow(unused)]
use super::*;

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct Preferences {
    pub use_default_accent_color: bool,
    pub accent_color: AccountAccentColor,
    pub show_separator: bool,
    pub multi_select: bool,
    pub save_on_pause: bool,
    pub show_body_border: bool,
}

impl Preferences {
    pub fn new(user: &UserSession) -> Self {
        let accent_color = AccountAccentColor::new(user);
        Self {
            use_default_accent_color: true,
            accent_color,
            show_separator: false,
            multi_select: false,
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
            multi_select: value.multi_select,
            save_on_pause: value.save_on_pause,
            show_body_border: value.show_body_border,
        }
    }
}

pub fn provide_prefs() -> Resource<Preferences> {
    let user = expect_context::<RwSignal<UserSession>>();

    let prefs_resource = Resource::new(user, move |user| async move {
        api::get_user_preferences(user).await.unwrap_or_default()
    });

    let owner = Owner::current().unwrap();
    owner.with(move || provide_context(prefs_resource));

    let prefs = RwSignal::new(Preferences::default());
    owner.with(move || provide_context(prefs));

    Effect::new_isomorphic(move || {
        if let Some(p) = prefs_resource.get() {
            prefs.set(p);
        }
    });

    prefs_resource
}
