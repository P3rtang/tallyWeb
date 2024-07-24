use std::collections::HashMap;

use chrono::Duration;
use leptos::*;

pub type MessageKey = usize;

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

pub trait Handle: Clone + Copy + 'static {}
#[derive(Debug, Clone, Copy)]
pub struct WithHandle;
impl Handle for WithHandle {}
#[derive(Debug, Clone, Copy)]
pub struct NoHandle;
impl Handle for NoHandle {}

#[derive(Debug, Clone, Copy)]
pub struct MessageJar<T: Handle> {
    messages: RwSignal<HashMap<MessageKey, Notification>>,
    reset_time: Option<Duration>,
    next_key: RwSignal<MessageKey>,
    as_modal: bool,
    phantomdata: std::marker::PhantomData<T>,
}

#[allow(dead_code)]
impl<T: Handle + 'static> MessageJar<T> {
    pub fn new(reset_time: Duration) -> Self {
        Self {
            messages: HashMap::new().into(),
            reset_time: Some(reset_time),
            as_modal: false,
            next_key: 0.into(),
            phantomdata: std::marker::PhantomData {},
        }
    }

    fn get_ordered(&self) -> Signal<Vec<(MessageKey, Notification)>> {
        create_read_slice(self.messages, |msgs| {
            let mut entries = msgs
                .iter()
                .map(|(key, value)| (*key, value.clone()))
                .collect::<Vec<_>>();
            entries.sort_by(|a, b| a.0.cmp(&b.0));
            entries
        })
    }

    pub fn is_emtpy(&self) -> bool {
        self.messages.get().is_empty()
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

    fn add_msg(&self, msg: NotificationKind) -> MessageKey {
        self.next_key.update(|k| *k += 1);
        let key = self.next_key.get_untracked();
        self.messages.update(|m| {
            m.insert(
                key,
                Notification {
                    kind: msg,
                    do_fade: false,
                },
            );
        });
        key
    }

    fn msg_timeout_effect(self, key: MessageKey) {
        if let Some(timeout) = self.reset_time {
            set_timeout(move || self.fade_out(key), timeout.to_std().unwrap())
        }
    }

    pub fn fade_out(self, key: MessageKey) {
        self.messages.update(|m| {
            if let Some(v) = m.get_mut(&key) {
                v.do_fade = true
            }
        })
    }

    pub fn get_last_key(self) -> Signal<MessageKey> {
        Signal::derive(self.next_key)
    }
}

impl MessageJar<NoHandle> {
    pub fn with_handle(self) -> MessageJar<WithHandle> {
        unsafe { std::mem::transmute(self) }
    }

    pub fn set_msg(self, msg: impl ToString) {
        let msg = msg.to_string();
        let msg_lines = msg.lines();
        let key = self.add_msg(NotificationKind::Message(
            self.as_modal,
            msg_lines
                .map(|l| view! { <b>{l.to_string()}</b> })
                .collect_view(),
        ));
        self.msg_timeout_effect(key);
    }

    pub fn set_msg_view(self, msg: impl IntoView + 'static) {
        let msg = msg.into_view();
        let key = self.add_msg(NotificationKind::Message(self.as_modal, msg.clone()));
        self.msg_timeout_effect(key);
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

    pub fn set_err(self, err: impl ToString) {
        let err = err.to_string();
        let msg_lines = err.lines();
        let key = self.add_msg(NotificationKind::Error(
            self.as_modal,
            msg_lines
                .map(|l| view! { <b>{l.to_string()}</b> })
                .collect_view(),
        ));
        self.msg_timeout_effect(key);
    }

    pub fn set_err_view(&self, err: impl IntoView) {
        let key = self.add_msg(NotificationKind::Error(self.as_modal, err.into_view()));
        self.msg_timeout_effect(key)
    }

    pub fn set_server_err(&self, err: &leptos::ServerFnError) {
        match err {
            ServerFnError::WrappedServerError(e) => self.set_err(e),
            ServerFnError::Registration(e) => self.set_err(e),
            ServerFnError::Request(e) => self.set_err(e),
            ServerFnError::Response(e) => self.set_err(e),
            ServerFnError::ServerError(e) => self.set_err(e),
            ServerFnError::Deserialization(e) => self.set_err(e),
            ServerFnError::Serialization(e) => self.set_err(e),
            ServerFnError::Args(e) => self.set_err(e),
            ServerFnError::MissingArg(e) => self.set_err(e),
        }
    }
}

impl MessageJar<WithHandle> {
    pub fn set_msg(self, msg: impl ToString) -> MessageKey {
        let msg = msg.to_string();
        let msg_lines = msg.lines();
        let key = self.add_msg(NotificationKind::Message(
            self.as_modal,
            msg_lines
                .map(|l| view! { <b>{l.to_string()}</b> })
                .collect_view(),
        ));
        self.msg_timeout_effect(key);
        key
    }

    pub fn set_msg_view(self, msg: impl IntoView + 'static) -> MessageKey {
        let msg = msg.into_view();
        let key = self.add_msg(NotificationKind::Message(self.as_modal, msg.clone()));
        self.msg_timeout_effect(key);
        key
    }

    pub fn set_success(&self, msg: &str) -> MessageKey {
        let msg_lines = msg.lines();
        let key = self.add_msg(NotificationKind::Success(
            self.as_modal,
            msg_lines
                .map(|l| view! { <b>{l.to_string()}</b> })
                .collect_view(),
        ));

        self.msg_timeout_effect(key);
        key
    }

    pub fn set_success_view(&self, msg: impl IntoView) -> MessageKey {
        let key = self.add_msg(NotificationKind::Success(self.as_modal, msg.into_view()));
        self.msg_timeout_effect(key);
        key
    }

    pub fn set_err(self, err: impl ToString) -> MessageKey {
        let err = err.to_string();
        let msg_lines = err.lines();
        let key = self.add_msg(NotificationKind::Error(
            self.as_modal,
            msg_lines
                .map(|l| view! { <b>{l.to_string()}</b> })
                .collect_view(),
        ));
        self.msg_timeout_effect(key);
        key
    }

    pub fn set_err_view(&self, err: impl IntoView) -> MessageKey {
        let key = self.add_msg(NotificationKind::Error(self.as_modal, err.into_view()));
        self.msg_timeout_effect(key);
        key
    }

    pub fn set_server_err(&self, err: &leptos::ServerFnError) -> MessageKey {
        match err {
            ServerFnError::WrappedServerError(e) => self.set_err(e),
            ServerFnError::Registration(e) => self.set_err(e),
            ServerFnError::Request(e) => self.set_err(e),
            ServerFnError::Response(e) => self.set_err(e),
            ServerFnError::ServerError(e) => self.set_err(e),
            ServerFnError::Deserialization(e) => self.set_err(e),
            ServerFnError::Serialization(e) => self.set_err(e),
            ServerFnError::Args(e) => self.set_err(e),
            ServerFnError::MissingArg(e) => self.set_err(e),
        }
    }
}

#[component]
fn Message(key: MessageKey, jar: MessageJar<NoHandle>) -> impl IntoView {
    if !jar.messages.get_untracked().contains_key(&key) {
        return view! {}.into_view();
    }

    let kind = create_read_slice(jar.messages, move |map| map.get(&key).unwrap().kind.clone());

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

    let dialog_ref = create_node_ref::<html::Dialog>();
    create_effect(move |_| {
        if let Some(d) = dialog_ref() {
            d.close();
            if is_modal() {
                let _ = d.show_modal();
            } else {
                d.show();
            }
        }
    });

    let dialog_class = create_read_slice(jar.messages, move |map| {
        if map.get(&key).unwrap().do_fade {
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
        jar.messages.update(|m| {
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
                {move || kind.get().get_view().unwrap_or(view! {}.into_view())}
            </div>
        </dialog>
    }
    .into_view()
}

#[component]
pub fn ProvideMessageSystem() -> impl IntoView {
    let msg_jar = MessageJar::new(Duration::seconds(5));
    provide_context(msg_jar);

    // on navigation clear any messages or errors from the message box
    // let loc_memo = create_memo(move |_| {
    //     let location = leptos_router::use_location();
    //     location.state.with(|_| msg_box.clear())
    // });
    //

    view! {
        <Show when=move || !msg_jar.is_emtpy()>
            <notification-box>
                <For
                    each=move || msg_jar.get_ordered().get().into_iter().rev()
                    key=|(key, _)| *key
                    children=move |(key, _)| {
                        view! { <Message key jar=msg_jar/> }
                    }
                />

            </notification-box>
        </Show>
    }
}
