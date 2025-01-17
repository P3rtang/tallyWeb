use super::*;
use leptos::attr::NextAttribute;

#[component]
pub fn BoolField(
    #[prop(into)] id: Signal<String>,
    #[prop(into, optional)] label: Option<Signal<String>>,
) -> impl IntoView {
    view! {
        <AttributeInterceptor let:attrs>
        {
            let attrs = view!{<{..} attr:id=id.get() {..attrs} />}.into_any_attr();
            view! {
                <Show when=move || label.is_some()>
                    <label for=id style:grid-column="1">
                        {label.unwrap()()}
                    </label>
                </Show>
                <div style:grid-column="2">
                    <Slider>
                        <InputSlot attrs slot/>
                    </Slider>
                </div>
            }
        }
        </AttributeInterceptor>
    }
}
