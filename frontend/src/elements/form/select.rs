use super::*;

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
    let screen = hooks::use_screen();
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

        view! { <Icon kind=IconKind::ArrowHeadDown color=IconColor::White style:transform=transform /> }
    };

    view! {
        // TODO: look into trying to make this a label
        <Show when=move || label.is_some()>
            <div class=style::form_label style:grid-column="1">
                {label.unwrap().get()}
            </div>
        </Show>
        <div
            class=stylance::classes!(style::select)
            style:grid-column=move || {
                if screen.get().viewport() > ViewPort::Small { "2" } else { "1" }
            }
        >
            <Select
                attr:id=id
                value
                options
                on_change=handle_change
                view=move |s| s.to_string().into_view()
            >
                <SelectInput attrs slot />
                <SelectButton
                    let:child
                    attrs=move || view! { <{..} class="hover-darken icon" /> }
                    slot
                >
                    {button_children(child)}
                </SelectButton>
            </Select>
        </div>
    }
}
