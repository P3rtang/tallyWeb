use super::*;

#[component]
pub fn ColorField(
    #[prop(into)] id: Signal<String>,
    #[prop(into, optional)] label: Option<Signal<String>>,
    #[prop(into, optional)] input_ref: NodeRef<html::Input>,
    #[prop(into, optional)] input_attrs: Option<AnyAttribute>,
) -> impl IntoView {
    let screen = hooks::use_screen();

    view! {
        <>
            <Show when=move || label.is_some()>
                <label for=id style:grid-column="1">
                    {label.unwrap()()}
                </label>
            </Show>
            <div
                class=style::color_input
                style:grid-column=move || {
                    if screen.get().viewport() > ViewPort::Small { "2" } else { "1" }
                }
            >
                <input
                    {..input_attrs.unwrap_or(().into_any_attr())}
                    node_ref=input_ref
                    id=id
                    type="color"
                />
            </div>
        </>
    }
}
