use super::*;

pub type MessageKey = usize;

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
    next_key: RwSignal<MessageKey>,
    as_modal: bool,
    phantomdata: std::marker::PhantomData<T>,
    config: Option<StoredValue<NotificationConfig>>,
}

#[allow(dead_code)]
impl<T: Handle + 'static> MessageJar<T> {
    pub fn new(reset_time: Duration) -> Self {
        Self {
            messages: RwSignal::new(HashMap::new()),
            as_modal: false,
            next_key: RwSignal::new(0),
            phantomdata: std::marker::PhantomData {},
            config: Default::default(),
        }
    }

    pub fn messages(&self) -> RwSignal<HashMap<MessageKey, Notification>> {
        self.messages
    }

    pub fn get_ordered(&self) -> Signal<Vec<MessageKey>> {
        create_read_slice(self.messages, |msgs| {
            let mut entries = msgs.iter().map(|(key, _)| *key).collect::<Vec<_>>();
            entries.sort();
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
            config: Some(StoredValue::new(NotificationConfig {
                fade: None,
                ..self.config.map(|c| c.get_value()).unwrap_or_default()
            })),
            ..self
        }
    }

    pub fn with_timeout(self, reset_time: chrono::TimeDelta) -> Self {
        Self {
            config: Some(StoredValue::new(NotificationConfig {
                fade: Some(reset_time),
                ..self.config.map(|c| c.get_value()).unwrap_or_default()
            })),
            ..self
        }
    }

    pub fn as_modal(self) -> Self {
        Self {
            as_modal: true,
            ..self
        }
    }

    fn add_msg(mut self, msg: NotificationKind) -> MessageKey {
        let config = self.config.take().unwrap_or_default().get_value();

        self.next_key.update(|k| *k += 1);
        let key = self.next_key.get_untracked();

        self.messages.update(|m| {
            m.insert(key, Notification { kind: msg, config });
        });

        key
    }

    fn msg_timeout_effect(self, key: MessageKey) {
        if let Some(timeout) = self.config.and_then(|c| c.get_value().fade) {
            set_timeout(move || self.fade_out(key), timeout.to_std().unwrap())
        }
    }

    pub fn fade_out(self, key: MessageKey) {
        self.messages.update(|m| {
            if let Some(v) = m.get_mut(&key) {
                v.config.do_fade = true
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

    pub fn with_config(mut self, config: NotificationConfig) -> Self {
        self.config = Some(StoredValue::new(config));
        self
    }

    pub fn set_msg(mut self, msg: impl ToString) {
        let msg = StoredValue::new(msg.to_string());
        let config = self.config.take();

        let key = self.add_msg(NotificationKind::Message(
            self.as_modal,
            ViewFn::from(move || {
                msg.get_value()
                    .lines()
                    .map(|l| view! { <b>{l.to_string()}</b> })
                    .collect_view()
                    .into_any()
            }),
        ));

        self.msg_timeout_effect(key);
    }

    pub fn set_msg_view(self, msg: ViewFn) {
        let key = self.add_msg(NotificationKind::Message(self.as_modal, msg));
        self.msg_timeout_effect(key);
    }

    pub fn set_success(&self, msg: impl ToString) {
        let msg = StoredValue::new(msg.to_string());
        let key = self.add_msg(NotificationKind::Success(
            self.as_modal,
            ViewFn::from(move || {
                msg.get_value()
                    .lines()
                    .map(|l| view! { <b>{l.to_string()}</b> })
                    .collect_view()
                    .into_any()
            }),
        ));

        self.msg_timeout_effect(key)
    }

    pub fn set_success_view(&self, msg: impl IntoView + Sync + Clone + 'static) {
        let msg = StoredValue::new(msg);
        let key = self.add_msg(NotificationKind::Success(
            self.as_modal,
            ViewFn::from(move || msg.get_value().into_view()),
        ));
        self.msg_timeout_effect(key);
    }

    pub fn set_err(self, err: impl ToString) {
        let err = StoredValue::new(err.to_string());
        let key = self.add_msg(NotificationKind::Error(
            self.as_modal,
            ViewFn::from(move || {
                err.get_value()
                    .lines()
                    .map(|l| view! { <b>{l.to_string()}</b> })
                    .collect_view()
                    .into_any()
            }),
        ));
        self.msg_timeout_effect(key);
    }

    pub fn set_err_view(&self, err: impl IntoView + Sync + Clone + 'static) {
        let err = StoredValue::new(err);
        let key = self.add_msg(NotificationKind::Error(
            self.as_modal,
            ViewFn::from(move || err.get_value().into_view()),
        ));
        self.msg_timeout_effect(key)
    }

    pub fn set_server_err(&self, err: &ServerFnError) {
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
        let msg = StoredValue::new(msg.to_string());
        let key = self.add_msg(NotificationKind::Message(
            self.as_modal,
            ViewFn::from(move || {
                msg.get_value()
                    .lines()
                    .map(|l| view! { <b>{l.to_string()}</b> })
                    .collect_view()
                    .into_any()
            }),
        ));
        self.msg_timeout_effect(key);
        key
    }

    pub fn set_msg_view(self, msg: ViewFn) -> MessageKey {
        let key = self.add_msg(NotificationKind::Message(self.as_modal, msg));
        self.msg_timeout_effect(key);
        key
    }

    pub fn set_success(&self, msg: impl ToString) -> MessageKey {
        let msg = StoredValue::new(msg.to_string());
        let key = self.add_msg(NotificationKind::Success(
            self.as_modal,
            ViewFn::from(move || {
                msg.get_value()
                    .lines()
                    .map(|l| view! { <b>{l.to_string()}</b> })
                    .collect_view()
            }),
        ));

        self.msg_timeout_effect(key);
        key
    }

    pub fn set_success_view(&self, msg: impl IntoView + Sync + Clone + 'static) -> MessageKey {
        let msg = StoredValue::new(msg);
        let key = self.add_msg(NotificationKind::Success(
            self.as_modal,
            ViewFn::from(move || msg.get_value()),
        ));
        self.msg_timeout_effect(key);
        key
    }

    pub fn set_err(self, err: impl ToString) -> MessageKey {
        let err = StoredValue::new(err.to_string());
        let key = self.add_msg(NotificationKind::Error(
            self.as_modal,
            ViewFn::from(move || {
                err.get_value()
                    .lines()
                    .map(|l| view! { <b>{l.to_string()}</b> })
                    .collect_view()
            }),
        ));
        self.msg_timeout_effect(key);
        key
    }

    pub fn set_err_view(&self, err: ViewFn) -> MessageKey {
        let key = self.add_msg(NotificationKind::Error(self.as_modal, err));
        self.msg_timeout_effect(key);
        key
    }

    pub fn set_server_err(&self, err: &ServerFnError) -> MessageKey {
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
