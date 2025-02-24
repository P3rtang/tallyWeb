use super::*;

stylance::import_style!(style, "button.module.scss");

#[derive(Default, Clone, Copy)]
pub enum ButtonSize {
    #[default]
    Default,
    Big,
    Small,
}

impl ButtonSize {
    fn into_class(self) -> &'static str {
        match self {
            ButtonSize::Default => style::default,
            ButtonSize::Big => style::big,
            ButtonSize::Small => style::small,
        }
    }
}

#[derive(Default, Clone, Copy)]
pub enum ButtonRounding {
    None,
    #[default]
    Semi,
    Full,
}

impl ButtonRounding {
    fn into_class(self) -> &'static str {
        match self {
            ButtonRounding::None => style::no_rounded,
            ButtonRounding::Semi => "",
            ButtonRounding::Full => style::full_rounded,
        }
    }
}

#[derive(Default, Clone, Copy)]
pub enum ButtonHover {
    #[default]
    Darken,
    Lighten,
}

impl ButtonHover {
    fn into_class(self) -> &'static str {
        match self {
            ButtonHover::Darken => style::darken,
            ButtonHover::Lighten => style::lighten,
        }
    }
}

// TODO: allow a toggle property
#[component]
pub fn Button(
    children: ChildrenFn,
    #[prop(into, optional)] href: Option<Signal<String>>,
    #[prop(into, optional)] size: ButtonSize,
    #[prop(into, optional)] rounding: ButtonRounding,
    #[prop(into, optional)] hover: ButtonHover,
    #[prop(into, optional)] node_ref: NodeRef<html::Button>,
    #[prop(into, optional)] class: Signal<String>,
) -> impl IntoView {
    let children = StoredValue::new(children);

    if let Some(href) = href {
        Either::Left(view! {
            <AttributeInterceptor let:attrs>
                <A href=href>
                    <Button
                        node_ref=node_ref
                        class=stylance::classes!(
                            style::button,
                            size.into_class(),
                            rounding.into_class(),
                            hover.into_class(),
                            class.get().as_str()
                        )
                        {..attrs}
                    >
                        <div>{children.get_value()()}</div>
                    </Button>
                </A>
            </AttributeInterceptor>
        })
    } else {
        Either::Right(view! {
            <button
                node_ref=node_ref
                class=stylance::classes!(
                    style::button,
                    size.into_class(),
                    rounding.into_class(),
                    hover.into_class(),
                    class.get().as_str()
                )
            >
                <div>{ children.get_value()() }</div>
            </button>
        })
    }
}
