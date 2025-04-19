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

    #[prop(optional)] children: Option<ChildrenFn>,
    #[prop(optional)] menu_entry_slot: MenuEntrySlot,
) -> impl IntoView {
    let menu_entry = StoredValue::new(menu_entry_slot);

    if let Some(children) = children {
        EitherOf3::A(view! {
            <div {..attrs.call()} class=style::entry>
                {children()}
            </div>
        })
    } else if let Some(href) = href {
        EitherOf3::B(view! {
            <Button
                class=style::entry
                href=href
                xstyle=xstyle!("padding": XPadding::Medium)
                {..attrs.call()}
            >
                <div class=style::label>
                    <Show when=move || menu_entry.get_value().icon.is_some()>
                        <Icon kind=menu_entry.get_value().icon.unwrap() />
                        <span>{menu_entry.get_value().label}</span>
                    </Show>
                </div>
            </Button>
        })
    } else {
        EitherOf3::C(view! {
            <Button class=style::entry xstyle=xstyle!("padding": XPadding::Medium) {..attrs.call()}>
                <div class=style::label>
                    <Show when=move || menu_entry.get_value().icon.is_some()>
                        <Icon kind=menu_entry.get_value().icon.unwrap() />
                        <span>{menu_entry.get_value().label}</span>
                    </Show>
                </div>
            </Button>
        })
    }
}

#[component]
pub fn MenuBreak() -> impl IntoView {
    view! { <hr /> }
}
