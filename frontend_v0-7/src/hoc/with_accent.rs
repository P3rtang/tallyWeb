use super::*;
use leptos::prelude::*;

pub fn with_accent(component: impl IntoView) -> impl IntoView {
    let prefs = use_context::<RwSignal<Preferences>>();

    let accent = move || {
        prefs
            .get()
            .map(|p| p.accent_color.0)
            .unwrap_or(Color::default().to_string())
    };

    let style = move || format!("--accent: {}", accent());

    component
        .into_view()
        .add_any_attr(view! { <{..} style=style /> })
}
