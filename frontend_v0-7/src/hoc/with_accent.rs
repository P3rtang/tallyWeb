use super::*;

pub fn with_accent<IV>(component: IV) -> impl IntoView
where
    IV: IntoView + Clone + Send + Sync + 'static,
{
    let prefs = use_context::<Resource<Preferences>>()
        .ok_or(crate::AppError::MissingPreferences(
            "with_accent".to_string(),
            ", use `with_accent_prefs` isntead".to_string(),
        ))
        .unwrap();

    let component = StoredValue::new(component);

    view! {
        <Transition fallback=move || component.get_value().add_any_attr(view! { <{..} style:--accent="#8BE9FD" /> })>
            {
                let accent = move || prefs.get().map(|p| p.accent_color.to_string()).unwrap_or("#8BE9FD".to_string());
                component.get_value().add_any_attr(view! { <{..} style:--accent=accent /> })
            }
        </Transition>
    }
}

pub fn with_accent_prefs(component: impl IntoView, prefs: RwSignal<Preferences>) -> impl IntoView {
    let accent = move || prefs.get().accent_color.0;

    component.add_any_attr(view! { <{..} style:--accent=accent /> })
}
