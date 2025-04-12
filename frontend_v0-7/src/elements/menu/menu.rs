use super::*;

// TODO: think about allowing custom offsets from the base menu position
#[component]
pub fn Menu(children: ChildrenFn, #[prop(optional)] menu_button: MenuButton) -> impl IntoView {
    let (overlay, _) = hooks::use_overlay().unwrap();
    let screen = hooks::use_screen();

    let node_ref = NodeRef::<html::Button>::new();

    let (vert_offset, set_vert_offset) = signal(0.0);
    let (horz_offset, set_horz_offset) = signal(0.0);
    let (height, set_height) = signal(0.0);
    let (width, set_width) = signal(0.0);

    Effect::new(move || {
        screen.track();

        if let Some(node) = node_ref.get() {
            let rect = node.get_bounding_client_rect();
            set_vert_offset.set(rect.y());
            set_horz_offset.set(rect.x());

            set_height.set(rect.height());
            set_width.set(rect.width());
        }
    });

    let prefs = expect_context::<RwSignal<Preferences>>();

    let menu = move || {
        let vert = move || {
            let screen = screen.get();

            let is_right_side = screen.width / 2.0 < horz_offset.get();
            let is_bottom_side = screen.height / 2.0 < vert_offset.get();

            if is_bottom_side {
                (
                    String::new(),
                    format!(
                        "{}px",
                        screen.height - vert_offset.get() - height.get() + 12.0
                    ),
                )
            } else {
                (
                    format!("{}px", vert_offset.get() + height.get() + 12.0),
                    String::new(),
                )
            }
        };

        let horz = move || {
            let screen = screen.get();

            let is_right_side = screen.width / 2.0 < horz_offset.get();
            let is_bottom_side = screen.height / 2.0 < vert_offset.get();

            if is_right_side {
                (
                    String::new(),
                    format!("{}px", screen.width - horz_offset.get() - width.get()),
                )
            } else {
                (
                    format!("{}px", horz_offset.get() + width.get()),
                    String::new(),
                )
            }
        };

        hoc::with_accent_prefs(
            view! {
                <div
                    style:top=move || vert().0
                    style:bottom=move || vert().1
                    style:left=move || horz().0
                    style:right=move || horz().1
                    class=style::menu
                >
                    {children()}
                </div>
            },
            prefs,
        )
    };

    let handle_click = move |ev: web_sys::MouseEvent| {
        ev.stop_propagation();
        overlay(menu.clone().into())
    };

    view! {
        <Button
            node_ref=node_ref
            xstyle=xstyle!("padding": XPadding::Medium)

            on:click={handle_click}
            {..menu_button.attrs.call()}
        >{(menu_button.children)()}</Button>
    }
}
