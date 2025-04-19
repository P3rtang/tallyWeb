use super::*;

stylance::import_style!(style, "./text.module.scss");

#[component]
pub fn Text(
    children: ChildrenFn,
    #[prop(optional)] fade: bool,
    #[prop(optional)] xstyle: Signal<XStyle>,
) -> impl IntoView {
    view! {
        <span
            class=stylance::classes!(style::text, fade.then_some(style::fade))
            {..xstyle.get().into_any_attr()}
        >
            {children()}
        </span>
    }
}
