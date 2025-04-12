use super::*;

stylance::import_style!(style, "./message.module.scss");

#[component]
pub fn Message(#[prop(name = "state")] MessageProp { key, jar, attrs }: MessageProp) -> AnyView {
    if !jar.messages().get_untracked().contains_key(&key) {
        return ().into_view().into_any();
    }

    let notification = move || jar.messages().get()[&key].clone();
    let kind = move || notification().kind.clone();

    let is_modal = move || match kind() {
        NotificationKind::Message(is_modal, _) => is_modal,
        NotificationKind::Error(is_modal, _) => is_modal,
        NotificationKind::Success(is_modal, _) => is_modal,
    };

    let dialog_ref = NodeRef::<html::Dialog>::new();
    Effect::new(move |_| {
        if let Some(d) = dialog_ref.get() {
            d.close();
            if is_modal() {
                let _ = d.show_modal();
            } else {
                d.show();
            }
        }
    });

    let on_close = move |ev: ev::MouseEvent| {
        ev.stop_propagation();
        jar.fade_out(key)
    };

    let on_fade_out = move |_| {
        jar.messages().update(|m| {
            m.remove(&key);
        })
    };

    let do_fade = move || notification().config.do_fade;

    let dialog_class = move || {
        stylance::classes!(
            style::message,
            if do_fade() { Some(style::fade) } else { None },
            match kind() {
                NotificationKind::Message(..) => style::info,
                NotificationKind::Error(..) => style::error,
                NotificationKind::Success(..) => style::success,
            }
        )
    };

    view! {
        <dialog
            node_ref=dialog_ref
            class=dialog_class
            on:animationend=on_fade_out
            on:click=|ev| ev.stop_propagation()

            {..attrs.call()}
        >
            <div class=style::content>
                <Button xstyle=xstyle!("padding": XPadding::Small) on:click=on_close>
                    <Icon kind=IconKind::Cross />
                </Button>
                {move || kind().get_view().unwrap_or(().into_any())}
            </div>
        </dialog>
    }
    .into_any()
}
