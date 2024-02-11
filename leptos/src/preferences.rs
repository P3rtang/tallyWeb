use leptos::*;
use serde::{Serialize, Deserialize};

use super::*;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountAccentColor(pub String);

impl AccountAccentColor {
    fn new(user: &UserSession) -> Self {
        let mut this = Self(String::new());
        this.set_user(user);
        this
    }

    pub fn set_user(&mut self, user: &UserSession) {
        let letter = user
            .username
            .to_uppercase()
            .chars()
            .next()
            .unwrap_or_default();
        let color_hex = components::letter_to_three_digit_hash(letter);
        self.0 = format!("#{color_hex}")
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
}

impl Preferences {
    pub fn new(user: &UserSession) -> Self {
        let accent_color = AccountAccentColor::new(user);
        Self {
            use_default_accent_color: true,
            accent_color,
            show_separator: false,
            multi_select: false,
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
        }
    }
}

#[component(transparent)]
pub fn ProvidePreferences(children: ChildrenFn) -> impl IntoView {
    let user = expect_context::<RwSignal<UserSession>>();
    let pref_signal = create_rw_signal(Preferences::new(&user.get_untracked()));
    provide_context(pref_signal);

    let pref_rsrc = create_blocking_resource(user, move |user| api::get_user_preferences(user));
    create_isomorphic_effect(move |_| {
        pref_rsrc.with(|p| {
            if let Some(Ok(p)) = p {
                pref_signal.set(p.clone())
            }
        });
    });

    view! {
        <Suspense fallback=move || view!{ <components::LoadingScreen/> }>
            {
                pref_rsrc.track();
                children()
            }
        </Suspense>
    }
}
