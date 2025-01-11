use super::*;

use components::MessageJar;
use leptos::server_fn::error::ServerFnErrorErr;

#[derive(Clone, Copy)]
pub struct ServerSaveHandler {}

impl ServerSaveHandler {
    pub fn new() -> Self {
        Self {}
    }
}

impl<S: ServerSavable + 'static> SaveHandler<S> for ServerSaveHandler {
    fn save(&self, value: S, on_error: ErrorFn) {
        if !value.has_change() {
            return;
        }

        #[allow(clippy::borrowed_box)]
        let action = Action::new(move |val: &Box<dyn ServerSavable>| val.save_endpoint());

        action.dispatch(Box::new(value));

        Effect::new(move |_| {
            match action.value()() {
                Some(Err(err)) => {
                    if !is_offline(&err) {
                        // msg.without_timeout().set_server_err(&err);
                        on_error(&ServerFnErrorErr::from(err))
                    }
                }
                _ => {}
            };
        });
    }
}

fn is_offline(err: &ServerFnError) -> bool {
    matches!(err, ServerFnError::Request(_))
}
