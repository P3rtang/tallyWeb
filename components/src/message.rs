use chrono::Duration;
use leptos::*;

#[derive(Debug, Clone, Copy)]
pub struct Message {
    msg: RwSignal<Notification>,
    reset_time: Option<Duration>,
    as_modal: bool,
}

#[allow(dead_code)]
impl Message {
    pub fn new(reset_time: Duration) -> Self {
        Self {
            msg: Notification::None.into(),
            reset_time: Some(reset_time),
            as_modal: false,
        }
    }

    pub fn clear(&self) {
        self.msg.set(Notification::None)
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
        return Self {
            as_modal: true,
            ..self
        };
    }

    pub fn get(&self) -> RwSignal<Notification> {
        self.msg
    }

    pub fn set_msg<'a>(&'a self, msg: &'a str) {
        let msg_lines = msg.lines();
        self.msg.set(Notification::Message(
            self.as_modal,
            msg_lines
                .map(|l| view! { <b>{ l.to_string() }</b> })
                .collect_view(),
        ));
        let msg = self.msg.clone();
        if let Some(reset_time) = self.reset_time {
            create_effect(move |_| {
                set_timeout(
                    move || msg.set(Notification::None),
                    reset_time.to_std().unwrap(),
                )
            });
        }
    }

    pub fn set_msg_view(&self, msg: impl IntoView) {
        self.msg
            .set(Notification::Message(self.as_modal, msg.into_view()));
        let msg = self.msg.clone();
        if let Some(reset_time) = self.reset_time {
            create_effect(move |_| {
                set_timeout(
                    move || msg.set(Notification::None),
                    reset_time.to_std().unwrap(),
                )
            });
        }
    }

    pub fn set_server_err<'a>(&self, err: &'a str) {
        let err_msg = err.split_once(": ").map(|s| s.1).unwrap_or(err);
        self.msg.set(Notification::Error(
            self.as_modal,
            view! { <b>{ err_msg.to_string() }</b> }.into_view(),
        ));
        let msg = self.msg.clone();
        if let Some(reset_time) = self.reset_time {
            create_effect(move |_| {
                set_timeout(
                    move || msg.set(Notification::None),
                    reset_time.to_std().unwrap(),
                )
            });
        }
    }

    pub fn set_err<'a>(&self, err: &'a str) {
        let msg_lines = err.lines();
        self.msg.set(Notification::Error(
            self.as_modal,
            msg_lines
                .map(|l| view! { <b>{ l.to_string() }</b> })
                .collect_view(),
        ));
        let msg = self.msg.clone();
        if let Some(reset_time) = self.reset_time {
            create_effect(move |_| {
                set_timeout(
                    move || msg.set(Notification::None),
                    reset_time.to_std().unwrap(),
                )
            });
        }
    }

    pub fn set_err_view(&self, err: impl IntoView) {
        self.msg
            .set(Notification::Error(self.as_modal, err.into_view()));
        let msg = self.msg.clone();
        if let Some(reset_time) = self.reset_time {
            create_effect(move |_| {
                set_timeout(
                    move || msg.set(Notification::None),
                    reset_time.to_std().unwrap(),
                )
            });
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Notification {
    None,
    Message(bool, View),
    Error(bool, View),
}

impl Notification {
    fn get_view(&self) -> Option<View> {
        match self {
            Notification::None => None,
            Notification::Message(_, msg) => Some(msg.clone()),
            Notification::Error(_, msg) => Some(msg.clone()),
        }
    }
}

#[component]
pub fn MessageBox(msg: Message) -> impl IntoView {
    let border_style = move || match msg.get()() {
        Notification::None => "",
        Notification::Message(_, _) => "border: 2px solid #ffe135",
        Notification::Error(_, _) => {
            "color: tomato;
            border: 2px solid tomato;"
        }
    };

    let is_modal = move || match msg.get()() {
        Notification::Message(is_modal, _) => is_modal,
        Notification::Error(is_modal, _) => is_modal,
        _ => false,
    };

    let dialog_ref = create_node_ref::<html::Dialog>();

    create_effect(move |_| match msg.get()() {
        Notification::None => {
            dialog_ref().map(|d| d.close());
        }
        _ if is_modal() => {
            dialog_ref().map(|d| d.close());
            dialog_ref().map(|d| d.show_modal());
        }
        _ => {
            dialog_ref().map(|d| d.close());
            dialog_ref().map(|d| d.show());
        }
    });

    view! {
        <dialog node_ref=dialog_ref class="notification-box" style=border_style>
            { move || { msg.get()().get_view().unwrap_or(view! {}.into_view()) } }
        </dialog>
    }
}
