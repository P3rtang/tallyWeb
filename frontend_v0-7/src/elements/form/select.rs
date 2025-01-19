use super::*;

stylance::import_style!(style, "./form.module.scss");

#[component]
pub fn SelectField<T>(
    #[prop(into, optional)] label: Option<Signal<String>>,
    #[prop(into, optional)] id: Option<Signal<String>>,
    #[prop(into)] options: Signal<Vec<T>>,
    #[prop(into, optional)] value: Option<Signal<T>>,
    #[prop(into, optional)] default_value: Option<T>,
    #[prop(into, optional)] on_change: EventCallback<Option<T>>,
    #[prop(into, optional)] attrs: AttributeFn,
) -> impl IntoView
where
    T: Sortable + ToString + PartialEq + Default + Clone + Send + Sync + 'static,
{
    let default_value = RwSignal::new(default_value);
    let value = Memo::new(move |_| {
        if value.is_some() {
            value.get().unwrap_or_default()
        } else {
            default_value.get().unwrap_or_default()
        }
    });

    let handle_change = move |t: Option<T>| {
        default_value.set(t.clone());
        on_change.call(t);
    };

    let button_children = move |s: SelectState<_>| {
        let transform = move || {
            if s.is_expanded {
                "rotate(180deg)"
            } else {
                ""
            }
        };

        view! {
            <Icon
                kind=IconKind::ArrowHeadDown
                color=IconColor::Black
                style:transform=transform
            />
        }
    };

    view! {
        <Show when=move || label.is_some()>
            <div id style:grid-column="1">{label.unwrap().get()}</div>
        </Show>
        <div class=stylance::classes!(style::select) style:grid-column="2">
            <Select
                value
                options
                on_change=handle_change
                view=move |s| s.to_string().into_view()
            >
                <SelectInput attrs slot />
                <SelectButton let:child attrs=move || view!{<{..} class="hover-darken icon" />} slot>
                    {button_children(child)}
                </SelectButton>
            </Select>
        </div>
    }
}
