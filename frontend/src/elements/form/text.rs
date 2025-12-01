use super::*;

#[derive(Clone, Copy)]
#[slot]
pub struct InputAttributes {
    attrs: StoredValue<AnyAttribute, LocalStorage>,
}

impl Default for InputAttributes {
    fn default() -> Self {
        Self {
            attrs: StoredValue::new_local(().into_any_attr()),
        }
    }
}

impl<A> From<A> for InputAttributes
where
    A: Attribute,
{
    fn from(value: A) -> Self {
        Self {
            attrs: StoredValue::new_local(value.into_any_attr()),
        }
    }
}

impl IntoAnyAttribute for InputAttributes {
    fn into_any_attr(self) -> AnyAttribute {
        self.attrs.get_value()
    }
}

#[component]
pub fn TextField(
    #[prop(into, optional)] id: Signal<Option<String>>,
    #[prop(into, optional)] label: Option<Signal<String>>,
    #[prop(into, default="text".into())] type_: Signal<String>,
    #[prop(into, optional)] input_ref: NodeRef<html::Input>,
    #[prop(optional, into)] input_attrs: InputAttributes,
) -> impl IntoView {
    let screen = hooks::use_screen();
    let text_align = move || match type_.get().as_str() {
        "number" => "end",
        _ => "unset",
    };

    view! {
        <Show when=move || label.is_some()>
            <label class=style::form_label for=id style:grid-column="1">
                {label.unwrap()()}
            </label>
        </Show>
        <div
            class=style::input
            style:grid-column=move || {
                if screen.get().viewport() > ViewPort::Small { "2" } else { "1" }
            }
        >
            <input
                node_ref=input_ref
                id=id
                style:text-align=text_align
                r#type=move || type_.get()
                {..input_attrs.into_any_attr()}
            />
        </div>
    }
}
