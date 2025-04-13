use super::*;

#[slot]
pub struct MenuButton {
    children: ChildrenFn,

    #[prop(into, optional)]
    attrs: AttributeFn,
}

impl Default for MenuButton {
    fn default() -> Self {
        let children = Arc::new(move || view! { <Icon kind=IconKind::Ellipsis /> }.into_any());

        Self {
            children,
            attrs: Default::default(),
        }
    }
}
