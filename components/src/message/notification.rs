use super::*;

#[derive(Clone, Copy)]
pub struct NotificationConfig {
    pub do_fade: bool,
    pub fade: Option<chrono::TimeDelta>,
    pub attrs: StoredValue<AnyAttribute, LocalStorage>,
}

impl NotificationConfig {
    pub fn new(fade: Option<chrono::TimeDelta>, attrs: impl Attribute) -> Self {
        Self {
            do_fade: false,
            fade,
            attrs: StoredValue::new_local(attrs.into_any_attr()),
        }
    }
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            do_fade: false,
            fade: None,
            attrs: StoredValue::new_local(().into_any_attr()),
        }
    }
}

impl std::fmt::Debug for NotificationConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "NotificationConfig: {{ do_fade: {}, fade: {:?}, attrs: StoredValue {{{:?}}} }}",
            self.do_fade,
            self.fade,
            self.attrs.get_value()
        )
    }
}

#[derive(Clone)]
pub struct Notification {
    pub kind: NotificationKind,
    pub config: NotificationConfig,
}

impl std::fmt::Debug for Notification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "kind: {}, do_fade: {}",
            match self.kind {
                NotificationKind::Message(_, _) => "Message",
                NotificationKind::Error(_, _) => "Error",
                NotificationKind::Success(_, _) => "Success",
            },
            self.config.do_fade
        )
    }
}

#[derive(Clone)]
pub enum NotificationKind {
    Message(bool, ViewFn),
    Error(bool, ViewFn),
    Success(bool, ViewFn),
}

unsafe impl Sync for NotificationKind {}
unsafe impl Send for NotificationKind {}

impl NotificationKind {
    pub fn get_view(&self) -> Option<AnyView> {
        match self {
            NotificationKind::Message(_, msg) => Some(msg.run()),
            NotificationKind::Error(_, msg) => Some(msg.run()),
            NotificationKind::Success(_, msg) => Some(msg.run()),
        }
    }
}
