use super::*;

stylance::import_style!(style, "./form.module.scss");

#[component]
pub fn TextField(
    #[prop(into)] id: Signal<String>,
    #[prop(into, optional)] label: Option<Signal<String>>,
    #[prop(into, default="text".into())] r#type: Signal<String>,
) -> impl IntoView {
    let text_align = move || match r#type.get().as_str() {
        "number" => "end",
        _ => "unset",
    };

    view! {
        <AttributeInterceptor let:attrs>
        {
            let align = view!{ <{..} style:text-align=text_align/> };

            view! {
                <Show when=move || label.is_some()>
                    <label for=id style:grid-column="1">
                        {label.unwrap()()}
                    </label>
                </Show>
                <div class=style::input style:grid-column="2">
                    <input id=move || id() r#type=move || r#type() {..align} {..attrs} />
                </div>
            }
        }
        </AttributeInterceptor>
    }
}
