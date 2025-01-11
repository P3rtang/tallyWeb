use super::*;

#[component]
pub fn TextField(
    #[prop(into)] id: Signal<String>,
    #[prop(into, optional)] label: Option<Signal<String>>,
) -> impl IntoView {
    view! {
        <AttributeInterceptor let:attrs>
            <Show when=move || label.is_some()>
                <label for=id style:grid-column="1">
                    {label.unwrap()()}
                </label>
            </Show>
            <div style:grid-column="2">
                <input {..attrs} />
            </div>
        </AttributeInterceptor>
    }
}
