use super::*;

stylance::import_style!(style, "button.module.scss");

#[component]
pub fn Button(children: ChildrenFn) -> impl IntoView {
    view! {
        <button class=style::button>
            <div>
            { children() }
            </div>
        </button>
    }
}
