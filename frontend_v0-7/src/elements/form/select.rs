use super::*;

stylance::import_style!(style, "./form.module.scss");

#[component]
pub fn SelectField<T>(
    #[prop(into, optional)] label: Option<Signal<String>>,
    #[prop(into, optional)] id: Option<Signal<String>>,
    #[prop(into)] options: Signal<Vec<T>>,
    #[prop(into, optional)] value: Option<Signal<T>>,
    #[prop(into, optional)] default_value: Option<T>,
    #[prop(into, optional)] attrs: AttributeFn,
) -> impl IntoView
where
    T: ToString + PartialEq + Default + Clone + Send + Sync + 'static,
{
    let selected = Memo::new(move |_| {
        if let Some(val) = value {
            val.get()
        } else {
            default_value.clone().unwrap_or_default()
        }
    });

    view! {
        <Show when=move || label.is_some()>
            <div id style:grid-column="1">{label.unwrap().get()}</div>
        </Show>
        <div class=stylance::classes!(style::select) style:grid-column="2">
            <Select
                selected
                options
                view=move |s| s.into_view()
            >
                <SelectInput attrs slot />
                <SelectButton attrs=move || view!{<{..} class="hover-darken icon" />} slot />
            </Select>
        </div>
    }
}
