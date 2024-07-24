use super::AppError;
use leptos::{create_action, create_effect, expect_context, view};

#[typetag::serde(tag = "type")]
pub trait Savable {
    fn indexed_db_name(self: &Self) -> String;
    fn save_endpoint(
        self: &Self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), leptos::ServerFnError>>>>;
}

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub struct SaveHandler {
    last_sync: chrono::NaiveDateTime,
}

impl SaveHandler {
    pub fn new() -> Self {
        Self {
            last_sync: chrono::Utc::now().naive_utc(),
        }
    }

    pub fn save(
        &self,
        value: Box<dyn Savable>,
        on_server_error: impl Fn() + 'static,
    ) -> Result<(), AppError> {
        self.save_server(value, on_server_error)
    }

    pub fn save_client(&self, _value: Box<dyn Savable>) -> Result<(), AppError> {
        Ok(())
    }

    pub fn save_server(
        &self,
        value: Box<dyn Savable>,
        on_error: impl Fn() + 'static,
    ) -> Result<(), AppError> {
        let msg = expect_context::<components::MessageJar>();

        let action = create_action(move |val: &Box<dyn Savable>| val.save_endpoint());
        action.dispatch(value);

        let msg_id = msg.with_handle().set_msg_view(view! {
            <div style="display: flex; align-items: center;">
                <components::Spinner></components::Spinner>
                <b style="font-size: 20px; padding-left: 24px;">Creating Counter</b>
            </div>
        });

        create_effect(move |_| {
            match action.value()() {
                Some(Err(err)) => {
                    msg.fade_out(msg_id);
                    if !is_offline(&err) {
                        msg.without_timeout().set_server_err(&err);
                        on_error()
                    } else {
                        msg.without_timeout()
                            .set_msg("Client offline\nWIP: offline modus");
                    }
                }
                Some(_) => msg.fade_out(msg_id),
                _ => {}
            };
        });

        Ok(())
    }
}

fn is_offline(err: &leptos::ServerFnError) -> bool {
    match err {
        leptos::ServerFnError::Request(_) => true,
        _ => false,
    }
}
