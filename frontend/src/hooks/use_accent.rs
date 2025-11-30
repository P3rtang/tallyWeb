use super::*;

pub fn use_accent() -> impl Attribute + Clone {
    let prefs = use_context::<RwSignal<Preferences>>()
        .ok_or(crate::AppError::MissingPreferences(
            "with_accent".to_string(),
            ", use `with_accent_prefs` isntead".to_string(),
        ))
        .unwrap();

    use_accent_prefs(prefs)
}

pub fn use_accent_prefs(prefs: RwSignal<Preferences>) -> impl Attribute + Clone {
    let accent = create_read_slice(prefs, |p| p.accent_color.to_string());

    view! {
        <{..} style:--accent=accent />
    }
}
