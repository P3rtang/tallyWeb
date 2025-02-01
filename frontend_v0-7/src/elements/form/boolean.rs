use super::*;
use leptos::attr::NextAttribute;

#[component]
pub fn BoolField(
    #[prop(into)] id: Signal<String>,
    #[prop(into, optional)] label: Option<Signal<String>>,
) -> impl IntoView {
    let screen = hooks::use_screen();

    view! {
        <AttributeInterceptor let:attrs>
        {
            let attrs = view!{<{..} attr:id=id.get() {..attrs} />}.into_any_attr();
            view! {
                <Show when=move || label.is_some()>
                    <label class=style::form_label for=id style:grid-column="1">
                        {label.unwrap()()}
                    </label>
                </Show>
                <div style:grid-column=move || { if screen.get().viewport() > ViewPort::Small { "2" } else { "1" } }>
                    <Slider>
                        <InputSlot attrs slot/>
                    </Slider>
                </div>
            }
        }
        </AttributeInterceptor>
    }
}
