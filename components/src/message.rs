use std::collections::HashMap;

use chrono::Duration;
use leptos::*;

#[derive(Debug, Clone, PartialEq)]
struct Notification {
    kind: NotificationKind,
    do_fade: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum NotificationKind {
    Message(bool, View),
    Error(bool, View),
    Success(bool, View),
}

impl NotificationKind {
    fn get_view(&self) -> Option<View> {
        match self {
            NotificationKind::Message(_, msg) => Some(msg.clone()),
            NotificationKind::Error(_, msg) => Some(msg.clone()),
            NotificationKind::Success(_, msg) => Some(msg.clone()),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MessageBox {
    messages: RwSignal<HashMap<usize, RwSignal<Notification>>>,
    reset_time: Option<Duration>,
    next_key: RwSignal<usize>,
    as_modal: bool,
}

#[allow(dead_code)]
impl MessageBox {
    pub fn new(reset_time: Duration) -> Self {
        Self {
            messages: HashMap::new().into(),
            reset_time: Some(reset_time),
            as_modal: false,
            next_key: 0.into(),
        }
    }

    fn get_ordered(&self) -> Signal<Vec<(usize, RwSignal<Notification>)>> {
        create_read_slice(self.messages, |msgs| {
            let mut entries = msgs
                .into_iter()
                .map(|(key, value)| (*key, value.clone()))
                .collect::<Vec<_>>();
            entries.sort_by(|a, b| a.0.cmp(&b.0));
            entries
        })
    }

    pub fn clear(&self) {
        self.messages.update(|list| list.clear())
    }

    pub fn without_timeout(self) -> Self {
        Self {
            reset_time: None,
            ..self
        }
    }

    pub fn with_timeout(self, reset_time: Duration) -> Self {
        Self {
            reset_time: Some(reset_time),
            ..self
        }
    }

    pub fn as_modal(self) -> Self {
        Self {
            as_modal: true,
            ..self
        }
    }

    fn add_msg(&self, msg: NotificationKind) -> usize {
        let key = self.next_key.get_untracked();
        self.messages.update(|m| {
            m.insert(
                key,
                Notification {
                    kind: msg,
                    do_fade: false,
                }
                .into(),
            );
        });
        self.next_key.update(|k| *k += 1);
        return key;
    }

    fn msg_timeout_effect(self, key: usize) {
        if let Some(timeout) = self.reset_time {
            create_effect(move |_| {
                set_timeout(move || self.fade_out(key), timeout.to_std().unwrap())
            });
        }
    }

    fn fade_out(self, key: usize) {
        self.messages.update(|m| {
            m.get_mut(&key).map(|v| v.update(|v| v.do_fade = true));
        })
    }

    pub fn set_msg(self, msg: impl ToString) {
        let msg = msg.to_string();
        create_effect(move |_| {
            let msg_lines = msg.lines();
            let key = self.add_msg(NotificationKind::Message(
                self.as_modal,
                msg_lines
                    .map(|l| view! { <b>{l.to_string()}</b> })
                    .collect_view(),
            ));
            self.msg_timeout_effect(key);
        });
    }

    pub fn set_msg_view(self, msg: impl IntoView + 'static) {
        let msg = msg.into_view();
        create_effect(move |_| {
            let key = self.add_msg(NotificationKind::Message(self.as_modal, msg.clone()));
            self.msg_timeout_effect(key);
        });
    }

    pub fn set_success(&self, msg: &str) {
        let msg_lines = msg.lines();
        let key = self.add_msg(NotificationKind::Success(
            self.as_modal,
            msg_lines
                .map(|l| view! { <b>{l.to_string()}</b> })
                .collect_view(),
        ));

        self.msg_timeout_effect(key)
    }

    pub fn set_success_view(&self, msg: impl IntoView) {
        let key = self.add_msg(NotificationKind::Success(self.as_modal, msg.into_view()));
        self.msg_timeout_effect(key);
    }

    pub fn set_server_err(&self, err: &str) {
        let err_msg = err.split_once(": ").map(|s| s.1).unwrap_or(err);
        let key = self.add_msg(NotificationKind::Error(
            self.as_modal,
            view! { <b>{err_msg.to_string()}</b> }.into_view(),
        ));
        self.msg_timeout_effect(key)
    }

    pub fn set_err(self, err: impl ToString) {
        let err = err.to_string();
        create_effect(move |_| {
            let msg_lines = err.lines();
            let key = self.add_msg(NotificationKind::Error(
                self.as_modal,
                msg_lines
                    .map(|l| view! { <b>{l.to_string()}</b> })
                    .collect_view(),
            ));
            self.msg_timeout_effect(key)
        });
    }

    pub fn set_err_view(&self, err: impl IntoView) {
        let key = self.add_msg(NotificationKind::Error(self.as_modal, err.into_view()));
        self.msg_timeout_effect(key)
    }
}

#[component]
fn Message(key: usize, #[prop(into)] msg: Signal<Notification>) -> impl IntoView {
    let msg_box = expect_context::<MessageBox>();

    let border_style = move || match msg().kind {
        NotificationKind::Message(_, _) => "border: 2px solid #ffe135",
        NotificationKind::Error(_, _) => {
            "color: tomato;
            border: 2px solid tomato;"
        }
        NotificationKind::Success(_, _) => {
            "color: #28a745;
            border: 2px solid #28a745;"
        }
    };

    let is_modal = move || match msg().kind {
        NotificationKind::Message(is_modal, _) => is_modal,
        NotificationKind::Error(is_modal, _) => is_modal,
        NotificationKind::Success(is_modal, _) => is_modal,
    };

    let dialog_ref = create_node_ref::<html::Dialog>();

    create_effect(move |_| match msg() {
        _ if is_modal() => {
            if let Some(d) = dialog_ref() {
                d.close();
                let _ = d.show_modal();
            }
        }
        _ => {
            if let Some(d) = dialog_ref() {
                d.close();
                d.show();
            }
        }
    });

    let dialog_class = create_memo(move |_| {
        if msg.get().do_fade {
            String::from("fade-out")
        } else {
            String::from("")
        }
    });

    let on_close_click = move |ev: ev::MouseEvent| {
        ev.stop_propagation();
        msg_box.fade_out(key)
    };

    let on_animend = move |_| {
        msg_box.messages.update(|m| {
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
                {move || { msg().kind.get_view().unwrap_or(view! {}.into_view()) }}
            </div>
        </dialog>
    }
}

#[component(transparent)]
pub fn ProvideMessageSystem() -> impl IntoView {
    let msg_box = MessageBox::new(Duration::seconds(5));
    provide_context(msg_box);

    // on navigation clear any messages or errors from the message box
    // let loc_memo = create_memo(move |_| {
    //     let location = leptos_router::use_location();
    //     location.state.with(|_| msg_box.clear())
    // });
    //

    view! {
        <notification-box>
            <For
                each=move || msg_box.get_ordered().get().into_iter().rev()
                key=|(key, _)| *key
                children=|(key, msg)| {
                    view! { <Message key msg/> }
                }
            />

        </notification-box>
    }
}
