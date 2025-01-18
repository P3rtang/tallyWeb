use super::*;

stylance::import_style!(style, "button.module.scss");

#[derive(Default)]
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

#[derive(Default)]
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

#[component]
pub fn Button(
    children: ChildrenFn,
    #[prop(into, optional)] size: ButtonSize,
    #[prop(into, optional)] rounding: ButtonRounding,
) -> impl IntoView {
    view! {
        <button class=stylance::classes!(style::button, size.into_class(), rounding.into_class())>
            <div>
            { children() }
            </div>
        </button>
    }
}
