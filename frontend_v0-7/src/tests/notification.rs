use super::*;

#[component]
pub fn TestNotifications() -> impl IntoView {
    let (fade, set_fade) = signal(None::<chrono::TimeDelta>);
    let (message_content, set_message_content) = signal("Message".to_string());
    let message = use_message();

    let config = move || {
        components::NotificationConfig::new(
            fade.get(),
            view! {
                <{..} data-testid="notification" />
            },
        )
    };

    let handle_click = move |_| {
        let msg = message_content.get_untracked();

        message(
            move || view! { <div>{msg.clone()}</div> },
            (fade.get(), config()).into(),
        )
    };

    let handle_error = move |_| {
        message(
            move || view! { <div>An Error Occured</div> },
            (Severity::Error, fade.get()).into(),
        )
    };

    let handle_change_text = move |ev| set_message_content.set(event_target_value(&ev));

    let handle_toggle_fade = move |_| {
        set_fade(if fade.get().is_some() {
            None
        } else {
            Some(chrono::TimeDelta::milliseconds(2000))
        })
    };

    view! {
        <div style:padding="32px">
            <Block
                spacing=48
                direction=block::Direction::Column
                style:align-items="start"
            >
                <Block spacing=16 direction=block::Direction::Column style:align-items="start">
                    <Block spacing=16 style:align-items="center">
                        <TextField attr:value=message_content on:change=handle_change_text attr:data-testid="message-input" />
                        <Slider on:change=handle_toggle_fade tooltip="Fade out" attr:data-testid="message-fade-slider"/>
                    </Block>
                    <Button on:click=handle_click attr:data-testid="send-message">Send message</Button>
                </Block>
                <Button on:click=handle_error attr:data-testid="send-error">Send Error</Button>
            </Block>
        </div>
    }
}
