use super::*;

#[slot]
pub struct MenuButton {
    children: ChildrenFn,

    #[prop(into)]
    attrs: AnyAttribute,
}

impl Default for MenuButton {
    fn default() -> Self {
        let children = Arc::new(move || view! { <Icon kind=IconKind::Ellipsis /> }.into_any());

        Self {
            children,
            attrs: ().into_any_attr(),
        }
    }
}
