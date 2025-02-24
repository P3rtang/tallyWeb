use super::*;

#[component]
pub fn TextField(
    #[prop(into)] id: Signal<String>,
    #[prop(into, optional)] label: Option<Signal<String>>,
    #[prop(into, default="text".into())] type_: Signal<String>,
    #[prop(into, optional)] input_ref: NodeRef<html::Input>,
) -> impl IntoView {
    let screen = hooks::use_screen();
    let text_align = move || match type_.get().as_str() {
        "number" => "end",
        _ => "unset",
    };

    view! {
        <AttributeInterceptor let:attrs>
        {
            let align = view!{ <{..} style:text-align=text_align/> };

            view! {
                <Show when=move || label.is_some()>
                    <label class=style::form_label for=id style:grid-column="1">
                        {label.unwrap()()}
                    </label>
                </Show>
                <div
                    class=style::input
                    style:grid-column=move || { if screen.get().viewport() > ViewPort::Small { "2" } else { "1" } }
                >
                    <input node_ref=input_ref id=id r#type=move || type_.get() {..align} {..attrs} />
                </div>
            }
        }
        </AttributeInterceptor>
    }
}
