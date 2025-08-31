use serde::de::DeserializeOwned;
use server_fn::{Http, ServerFn, client::Client, codec::PostUrl, request::ClientReq};
use web_sys::FormData;

use super::*;

#[component]
pub fn Form<ServFn, OutputProtocol>(
    action: ServerAction<ServFn>,
    children: ChildrenFn,
    #[prop(into, optional)] on_undo: EventCallback<ev::MouseEvent>,
    #[prop(into, optional)] id: Option<String>,
    #[prop(into, optional)] session: Option<Signal<UserSession>>,

    #[prop(optional)] header_slot: HeaderSlot,
) -> impl IntoView
where
    ServFn: DeserializeOwned
        + ServerFn<Protocol = Http<PostUrl, OutputProtocol>>
        + Clone
        + Send
        + Sync
        + 'static,
    <<ServFn::Client as Client<ServFn::Error>>::Request as ClientReq<ServFn::Error>>::FormData:
        From<FormData>,
    ServFn: Send + Sync + 'static,
    ServFn::Output: Send + Sync + 'static,
    ServFn::Error: Send + Sync + 'static,
    <ServFn as ServerFn>::Client: Client<<ServFn as ServerFn>::Error>,
{
    view! {
        <div class=style::container>
            {header_slot} <ActionForm action attr:class=style::form attr:id=id>
                <Show when=move || session.is_some()>
                    <session::SessionFormInput session=session.unwrap() />
                </Show>
                <div class=style::body>{children()}</div>
                <div class=style::action_buttons>
                    <div></div>
                    <div>
                        <button
                            type="button"
                            class="hover-darken"
                            on:click=move |ev| on_undo.call(ev)
                        >
                            <div>Undo</div>
                        </button>
                        <button
                            type="submit"
                            class=stylance::classes!(style::confirm, "hover-darken")
                        >
                            <div>Submit</div>
                        </button>
                    </div>
                </div>
            </ActionForm>
        </div>
    }
}
