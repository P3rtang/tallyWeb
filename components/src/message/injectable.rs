use super::*;

#[derive(Debug, Clone)]
pub struct MessageProp {
    pub key: MessageKey,
    pub jar: MessageJar,
    pub attrs: AttributeFn,
}

#[derive(Clone)]
#[slot]
pub struct MessageSlot {
    #[prop(into)]
    children: ChildrenPropFn<MessageProp>,
}

impl Default for MessageSlot {
    fn default() -> Self {
        Self {
            children: (move |MessageProp { key, jar, attrs }| {
                view! { <Message key jar attrs /> }.into_any()
            })
            .into(),
        }
    }
}

#[component]
pub fn Message(key: MessageKey, jar: MessageJar, attrs: AttributeFn) -> AnyView {
    if !jar.messages().get_untracked().contains_key(&key) {
        return ().into_view().into_any();
    }

    let kind = move || jar.messages()().get(&key).unwrap().kind.clone();

    let border_style = move || match kind() {
        NotificationKind::Message(_, _) => "border: 2px solid #ffe135",
        NotificationKind::Error(_, _) => "color: tomato; border: 2px solid tomato;",
        NotificationKind::Success(_, _) => "color: #28a745; border: 2px solid #28a745;",
    };

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

    let dialog_class = create_read_slice(jar.messages(), move |map| {
        if map.get(&key).unwrap().config.do_fade {
            String::from("fade-out")
        } else {
            String::from("")
        }
    });

    let on_close_click = move |ev: ev::MouseEvent| {
        ev.stop_propagation();
        jar.fade_out(key)
    };

    let on_animend = move |_| {
        jar.messages().update(|m| {
            m.remove(&key);
        })
    };

    view! {
        <dialog
            on:click=|ev| ev.stop_propagation()
            node_ref=dialog_ref
            class=dialog_class
            style=border_style
            on:animationend=on_animend
        >
            <div class="content">
                <button class="close" on:click=on_close_click>
                    <i class="fa-solid fa-xmark"></i>
                </button>
                {move || kind().get_view().unwrap_or(().into_any())}
            </div>
        </dialog>
    }
    .into_any()
}

#[component]
pub fn ProvideMessageJar(
    owner: Owner,
    #[prop(optional)] message_slot: MessageSlot,
) -> impl IntoView {
    let msg_jar = MessageJar::new(Duration::seconds(5));
    owner.with(|| provide_context(msg_jar));

    // on navigation clear any messages or errors from the message box
    Effect::new_sync(move |_| {
        let location = leptos_router::hooks::use_location();
        location.state.with(|_| msg_jar.clear())
    });

    let msg_view_fn = message_slot.children;

    view! {
        <notification-box>
            <For
                each=move || msg_jar.get_ordered().get().into_iter().rev()
                key=|key| *key
                children=move |key| {
                    let attrs = msg_jar
                        .messages()
                        .get()
                        .get(&key)
                        .map(|m| m.config.attrs)
                        .unwrap_or_default()
                        .get_value();
                    msg_view_fn
                        .clone()(MessageProp {
                        key,
                        jar: msg_jar,
                        attrs,
                    })
                }
            />
        </notification-box>
    }
}
