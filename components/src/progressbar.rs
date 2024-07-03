use leptos::*;

#[component]
pub fn Progressbar<F, C>(
    progress: F,
    color: C,
    #[prop(attrs)] attrs: Vec<(&'static str, Attribute)>,
    children: ChildrenFn,
) -> impl IntoView
where
    F: Fn() -> f64 + Copy + 'static,
    C: Fn() -> &'static str + Copy + 'static,
{
    view! {
        <progress-bar
            {..attrs}
            style:display="flex"
            style:justify-content="center"
            style:align-items="center"
        >
            <div
                style:font-size="1.4rem"
                style="
                    padding: 0px 12px;
                    margin: auto;"
                    .to_string()
            >
                {children()}
            </div>
            <through-bar style:width="100%" style:min-height="8px">
                <Show
                    when=move || { progress() > 0.0 }
                    fallback=move || {
                        view! {
                            <color-bar style:display="block" style:min-height="8px"></color-bar>
                        }
                    }
                >

                    <color-bar style=move || {
                        "display: block;".to_string() + "min-height: 8px;"
                            + format!("background: {};", color()).as_str()
                            + format!("width: max({}%, 10px);", progress() * 100.0).as_str()
                    }></color-bar>

                </Show>
            </through-bar>
        </progress-bar>
    }
}
