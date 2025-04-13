use super::*;

stylance::import_style!(style, "button.module.scss");

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
    #[prop(into, optional)] hover: ButtonHover,
    #[prop(into, optional)] node_ref: NodeRef<html::Button>,
    #[prop(into, optional)] class: Signal<String>,
    #[prop(into, optional)] xstyle: XStyle,
) -> impl IntoView {
    let children = StoredValue::new(children);

    if let Some(href) = href {
        Either::Left(view! {
            <AttributeInterceptor let:attrs>
                <A href=href attr:tabindex=-1>
                    <Button
                        xstyle
                        node_ref=node_ref
                        class=stylance::classes!(
                            style::button,
                            hover.into_class(),
                            class.get().as_str()
                        )
                        {..attrs}
                    >
                        {children.get_value()()}
                    </Button>
                </A>
            </AttributeInterceptor>
        })
    } else {
        Either::Right(view! {
            <AttributeInterceptor let:attrs>
                <button
                    node_ref=node_ref
                    class=stylance::classes!(
                        style::button,
                        hover.into_class(),
                        class.get().as_str()
                    )
                    {..xstyle.border_radius()}
                    {..attrs}
                >
                    <div {..xstyle.into_any_attr()}>{children.get_value()()}</div>
                </button>
            </AttributeInterceptor>
        })
    }
}
