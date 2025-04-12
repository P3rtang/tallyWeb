use super::*;

#[derive(Clone, Default)]
#[slot]
pub struct MenuEntrySlot {
    #[prop(optional)]
    icon: Option<IconKind>,

    #[prop(into)]
    label: String,

    #[prop(into, optional)]
    attrs: AttributeFn,
}

#[component]
pub fn MenuEntry(
    #[prop(into, optional)] href: Option<Signal<String>>,
    #[prop(into, optional)] attrs: AttributeFn,

    #[prop(into, optional)] children: Option<ChildrenFn>,
    #[prop(optional)] menu_entry_slot: MenuEntrySlot,
) -> impl IntoView {
    let menu_entry = StoredValue::new(menu_entry_slot);

    let children = StoredValue::new(children.unwrap_or(Arc::new(move || {
        {
            view! {
                <div class=style::label>
                    <Show when=move || menu_entry.get_value().icon.is_some()>
                        <Icon kind=menu_entry.get_value().icon.unwrap() />
                        <span>{menu_entry.get_value().label}</span>
                    </Show>
                </div>
            }
        }
        .into_any()
    })));

    let button_element = move || {
        if let Some(href) = href.get() {
            Either::Left(view! {
                <A href=href.clone() >
                    <Button {..attrs.call()} class=style::entry xstyle=xstyle!("padding": XPadding::Medium)>
                        {children.get_value()()}
                    </Button>
                </A>
            })
        } else {
            Either::Right(view! {
                <Button {..attrs.call()} class=style::entry xstyle=xstyle!("padding": XPadding::Medium)>
                    {children.get_value()()}
                </Button>
            })
        }
    };

    button_element()
}

#[component]
pub fn MenuBreak() -> impl IntoView {
    view! { <hr/> }
}
