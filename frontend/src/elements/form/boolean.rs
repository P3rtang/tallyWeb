use super::*;

#[component]
pub fn BoolField(
    #[prop(into)] id: Signal<String>,
    #[prop(into, optional)] label: Option<Signal<String>>,
    #[prop(into, optional)] input_attrs: Option<AnyAttribute>,
) -> impl IntoView {
    let screen = hooks::use_screen();

    view! {
        <>
            <Show when=move || label.is_some()>
                <label class=style::form_label for=id style:grid-column="1">
                    {label.unwrap()()}
                </label>
            </Show>
            <div style:grid-column=move || {
                if screen.get().viewport() > ViewPort::Small { "2" } else { "1" }
            }>
                <Slider input_attrs prop:id=id />
            </div>
        </>
    }
}
