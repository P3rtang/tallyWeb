use leptos::*;

#[component]
pub fn ToolTip<T: html::ElementDescriptor + Clone + 'static>(
    parent_node: NodeRef<T>,
    #[prop(optional, default=std::time::Duration::from_secs(1))] delay: std::time::Duration,
    children: ChildrenFn,
) -> impl IntoView {
    let is_shown = create_rw_signal(false);
    let is_hovering = create_rw_signal(false);
    let mouse_pos = create_rw_signal((0, 0));

    if let Some(element) = parent_node.get_untracked() {
        let _ = element.clone().on(ev::mouseover, move |_: ev::MouseEvent| {
            is_hovering.set(true);
            set_timeout(
                move || {
                    if is_hovering.try_get().unwrap_or_default() {
                        is_shown.try_set(true);
                    }
                },
                delay,
            )
        });
        let _ = element.clone().on(ev::mouseout, move |_: ev::MouseEvent| {
            is_hovering.set(false);
            is_shown.set(false);
        });
        let _ = element.on(ev::mousemove, move |ev: ev::MouseEvent| {
            if !is_shown() {
                mouse_pos.set((ev.x(), ev.y()))
            }
        });
    }

    view! {
        <Show when=is_shown>
            <tool-tip
                style:left=move || format!("{}px", mouse_pos().0 + 8)
                style:top=move || format!("{}px", mouse_pos().1 + 16)
            >
                {children()}
            </tool-tip>
        </Show>
    }
}
