use super::*;

#[derive(Debug, Clone, Copy)]
pub enum Severity {
    Info,
    Warning,
    Error,
    Success,
}

#[derive(Debug, Clone)]
pub struct MessageConfig {
    severity: Severity,
    config: NotificationConfig,
}

impl Default for MessageConfig {
    fn default() -> Self {
        Self {
            severity: Severity::Info,
            config: Default::default(),
        }
    }
}

impl From<Severity> for MessageConfig {
    fn from(value: Severity) -> Self {
        Self {
            severity: value,
            ..Default::default()
        }
    }
}

impl From<NotificationConfig> for MessageConfig {
    fn from(value: NotificationConfig) -> Self {
        Self {
            config: value,
            ..Default::default()
        }
    }
}

impl From<(Severity, NotificationConfig)> for MessageConfig {
    fn from(value: (Severity, NotificationConfig)) -> Self {
        Self {
            severity: value.0,
            config: value.1,
        }
    }
}

pub type DynMessageFn = Arc<dyn Fn(ViewFn, MessageConfig) -> usize + Send + Sync>;

/**
   # usage
   ```rust,ignore
   let message = use_message();

   let view_fn = move || view!{<div>Hello, world!</div>};

   let notification_config = NotificationConfig::new(
       None, // disable the fade timeout
       view!{<{..} attr:id="notification" />}, // add any attribute/style to the notification
   );

   let config = (
       Severity::Error, // set the severity to error (red border and text)
       notification_config, // object with styling and fading for the notification itself
   );

   message(view_fn, config.into());
   ```
*/
#[derive(Clone, Copy)]
pub struct MessageFn(Option<StoredValue<DynMessageFn>>);

impl MessageFn {
    pub fn server_err(&self, error: ServerFnError) -> Option<MessageKey> {
        if let Some(func) = self.0 {
            let msg: ViewFn = match error {
                #[allow(deprecated)]
                ServerFnError::WrappedServerError(_) => todo!(),
                ServerFnError::Registration(_) => todo!(),
                ServerFnError::Request(_) => {
                    (move || view! { <div>Failed to connect to the server</div> }).into()
                }
                ServerFnError::Response(_) => {
                    (move || view! { <div>The server failed to respond</div> }).into()
                }
                ServerFnError::ServerError(err) => (move || {
                    err.split('\n')
                        .map(|line| view! { <div>{line.to_string()}</div> })
                        .collect_view()
                })
                .into(),
                ServerFnError::Deserialization(_) => todo!(),
                ServerFnError::Serialization(_) => todo!(),
                ServerFnError::Args(_) => todo!(),
                ServerFnError::MissingArg(_) => todo!(),
                ServerFnError::MiddlewareError(_) => todo!(),
            };

            Some(func.get_value()(msg, (Severity::Error).into()))
        } else {
            None
        }
    }
}

impl FnOnce<(ViewFn, MessageConfig)> for MessageFn {
    type Output = ();

    extern "rust-call" fn call_once(self, args: (ViewFn, MessageConfig)) -> Self::Output {
        if let Some(func) = self.0 {
            func.get_value()(args.0, args.1);
        }
    }
}

impl Fn<(ViewFn, MessageConfig)> for MessageFn {
    extern "rust-call" fn call(&self, args: (ViewFn, MessageConfig)) -> Self::Output {
        if let Some(func) = &self.0 {
            func.get_value()(args.0, args.1);
        }
    }
}

impl FnMut<(ViewFn, MessageConfig)> for MessageFn {
    extern "rust-call" fn call_mut(&mut self, args: (ViewFn, MessageConfig)) -> Self::Output {
        if let Some(func) = &self.0 {
            func.get_value()(args.0, args.1);
        }
    }
}

impl<VF, IV> FnOnce<(VF, MessageConfig)> for MessageFn
where
    VF: Fn() -> IV + Send + Sync + 'static,
    IV: IntoView + Send + Sync + 'static,
{
    type Output = ();

    extern "rust-call" fn call_once(self, args: (VF, MessageConfig)) -> Self::Output {
        if let Some(func) = self.0 {
            func.get_value()(args.0.into(), args.1);
        }
    }
}

impl<VF, IV> Fn<(VF, MessageConfig)> for MessageFn
where
    VF: Fn() -> IV + Send + Sync + 'static,
    IV: IntoView + Send + Sync + 'static,
{
    extern "rust-call" fn call(&self, args: (VF, MessageConfig)) -> Self::Output {
        if let Some(func) = &self.0 {
            func.get_value()(args.0.into(), args.1);
        }
    }
}

impl<VF, IV> FnMut<(VF, MessageConfig)> for MessageFn
where
    VF: Fn() -> IV + Send + Sync + 'static,
    IV: IntoView + Send + Sync + 'static,
{
    extern "rust-call" fn call_mut(&mut self, args: (VF, MessageConfig)) -> Self::Output {
        if let Some(func) = &self.0 {
            func.get_value()(args.0.into(), args.1);
        }
    }
}

impl From<Option<(Arc<dyn Fn(ViewFn, MessageConfig) -> usize + Send + Sync>)>> for MessageFn {
    fn from(value: Option<(Arc<dyn Fn(ViewFn, MessageConfig) -> usize + Send + Sync>)>) -> Self {
        Self(value.map(|v| StoredValue::new(v)))
    }
}

impl<F> From<F> for MessageFn
where
    F: Fn(ViewFn, MessageConfig) -> usize + Send + Sync + 'static,
{
    fn from(value: F) -> Self {
        Self(Some(StoredValue::new(Arc::new(value))))
    }
}

pub fn use_message() -> MessageFn {
    #[cfg(feature = "ssr")]
    return MessageFn(None);

    use_context::<MessageJar>()
        .map(
            |jar| -> Arc<dyn Fn(ViewFn, MessageConfig) -> usize + Send + Sync> {
                Arc::new(move |msg: ViewFn, config: MessageConfig| {
                    let jar = if let Some(timeout) = config.config.fade {
                        jar.with_config(config.config)
                            .with_handle()
                            .with_timeout(timeout)
                    } else {
                        jar.with_config(config.config)
                            .with_handle()
                            .without_timeout()
                    };

                    match config.severity {
                        Severity::Error => jar.set_err_view(msg),
                        Severity::Warning => todo!(),
                        Severity::Info => jar.set_msg_view(msg),
                        Severity::Success => jar.set_success_view(msg),
                    }
                })
            },
        )
        .into()
}
