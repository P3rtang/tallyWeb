use chrono::Duration;
use leptos::*;

#[derive(Debug, Clone, Copy)]
pub struct Message {
    msg: RwSignal<Notification>,
    reset_time: Duration,
}

impl Message {
    pub fn new(reset_time: Duration) -> Self {
        Self {
            msg: Notification::None.into(),
            reset_time,
        }
    }

    pub fn get(&self) -> RwSignal<Notification> {
        self.msg
    }

    pub fn set_message<'a>(&'a self, msg: &'a str) {
        self.msg.set(Notification::Message(msg.to_string()));
        let msg = self.msg.clone();
        let reset_time = self.reset_time.clone();
        create_effect(move |_| {
            set_timeout(
                move || msg.set(Notification::None),
                reset_time.to_std().unwrap(),
            )
        });
    }

    pub fn set_error<'a>(&self, err: &'a str) {
        self.msg.set(Notification::Error(err.to_string()));
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Notification {
    None,
    Message(String),
    Error(String),
}

impl Notification {
    fn is_some(&self) -> bool {
        self != &Self::None
    }

    fn get_text(&self) -> Option<String> {
        match self {
            Notification::None => None,
            Notification::Message(msg) => Some(msg.to_string()),
            Notification::Error(msg) => Some(msg.to_string()),
        }
    }
}

#[component]
pub fn MessageBox(msg: Message) -> impl IntoView {
    let border_style = move || match msg.get()() {
        Notification::None => "",
        Notification::Message(_) => "border: 2px solid #ffe135",
        Notification::Error(_) => {
            "color: tomato;
            border: 2px solid tomato;"
        }
    };

    view! {
        <Show
            when=move || { msg.get()().is_some() }
                fallback=|| ()
        >
            <b class="notification-box" style=border_style>{ move || { msg.get()().get_text().unwrap() } }</b>
        </Show>
    }
}
