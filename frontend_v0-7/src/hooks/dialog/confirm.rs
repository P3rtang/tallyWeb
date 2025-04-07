use super::*;

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
            log!("in waker");
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

pub fn use_confirm() -> impl Fn(String) -> ConfirmHandler {
    // TODO: unable use without page context provide fallback
    let page_context = use_context::<PageContext>().unwrap();
    let mut resolve = false;

    move |title: String| {
        let handler = ConfirmHandler::default();
        let clone = handler.clone();

        let handle_confirm = move || clone.confirm();

        page_context.overlay.body.set((move || view! {<ConfirmContent on_confirm=handle_confirm.clone() title=title.clone() />}).into());
        page_context.overlay.is_open.set(true);

        handler
    }
}

#[component]
fn ConfirmContent<C>(on_confirm: C, title: String) -> impl IntoView
where
    C: Fn() + Send + Sync + 'static,
{
    let handle_confirm = move |_| on_confirm();

    view! {
        <div class=style::container>
            <div>{title.clone()}</div>
            <div class=style::actions>
                <Button class=style::main on:click={handle_confirm}>Ok</Button>
            </div>
        </div>
    }
}
