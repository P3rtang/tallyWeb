use super::*;

pub enum Reason {
    Background,
    CancelButton,
    OkButton,
}

#[derive(Default, Clone)]
pub struct ConfirmHandlerState {
    waker: Option<Waker>,
    completed: bool,
    confirm: bool,
}

#[derive(Default, Clone)]
pub struct ConfirmHandler {
    state: Arc<Mutex<ConfirmHandlerState>>,
}

impl ConfirmHandler {
    fn confirm(&self) {
        let mut state = self.state.lock().unwrap();
        state.confirm = true;
        state.completed = true;

        if let Some(waker) = state.waker.take() {
            waker.wake();
        }
    }

    fn cancel(&self) {
        let mut state = self.state.lock().unwrap();
        state.completed = true;

        if let Some(waker) = state.waker.take() {
            waker.wake();
        }
    }
}

impl Future for ConfirmHandler {
    type Output = bool;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let mut state = self.state.lock().unwrap();

        if state.completed {
            Poll::Ready(state.confirm)
        } else {
            state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

pub struct Canceled {}

pub fn use_confirm() -> impl Fn(String) -> std::pin::Pin<Box<dyn Future<Output = bool>>> {
    // TODO: unable use without page context provide fallback
    let page_context = use_context::<PageContext>().unwrap();
    let preferences = expect_context::<RwSignal<Preferences>>();
    let mut resolve = false;

    move |title: String| {
        let handler = StoredValue::new(ConfirmHandler::default());

        let on_event = move |_, reason: Reason| match reason {
            Reason::Background => handler.get_value().cancel(),
            Reason::CancelButton => handler.get_value().cancel(),
            Reason::OkButton => handler.get_value().confirm(),
        };

        page_context.dialog.show(
            move || {
                view! { <ConfirmContent on_event title=title.clone() prefs=preferences /> }
            },
            move |ev: MouseEvent| on_event(ev, Reason::Background),
        );

        handler
            .get_value()
            .then(async move |ok| {
                if (!ok) {
                    page_context.dialog.open.set(false)
                }

                ok
            })
            .boxed()
    }
}

#[component]
fn ConfirmContent<CO>(on_event: CO, title: String, prefs: RwSignal<Preferences>) -> impl IntoView
where
    CO: Fn(MouseEvent, Reason) + Send + Sync + Clone + 'static,
{
    let on_event = StoredValue::new(move |ev: MouseEvent, reason| {
        ev.stop_propagation();
        on_event(ev, reason)
    });

    let handle_background = move |ev: leptos::ev::MouseEvent| ev.stop_propagation();
    let handle_cancel = move |ev| on_event.get_value()(ev, Reason::CancelButton);
    let handle_confirm = move |ev| on_event.get_value()(ev, Reason::OkButton);

    hoc::with_accent_prefs(
        move || {
            view! {
                <div class=style::container on:click=handle_background>
                    <h2>{title.clone()}</h2>
                    <div class=style::content></div>
                    <div class=style::actions>
                        <Button on:click=handle_cancel>Cancel</Button>
                        <Button class=style::main on:click=handle_confirm>
                            Ok
                        </Button>
                    </div>
                </div>
            }
        },
        prefs,
    )
}
