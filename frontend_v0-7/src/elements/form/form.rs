use serde::de::DeserializeOwned;
use server_fn::{client::Client, codec::PostUrl, request::ClientReq, ServerFn};
use web_sys::FormData;

use super::*;

#[component]
pub fn Form<ServFn>(
    action: ServerAction<ServFn>,
    children: ChildrenFn,
    #[prop(into, optional)] on_undo: EventCallback<ev::MouseEvent>,
    #[prop(into, optional)] on_submit: EventCallback<ev::SubmitEvent>,

    #[prop(optional)] header_slot: HeaderSlot,
) -> impl IntoView
where
    ServFn: DeserializeOwned + ServerFn<InputEncoding = PostUrl> + Clone + Send + Sync + 'static,
    <<ServFn::Client as Client<ServFn::Error>>::Request as ClientReq<ServFn::Error>>::FormData:
        From<FormData>,
    ServFn::Output: Send + Sync + 'static,
    ServFn::Error: Send + Sync + 'static,
{
    view! {
        <div class=style::container>
            { header_slot }
            <ActionForm action attr:class=style::form>
                <div class=style::body>
                    {children()}
                </div>
                <div class=style::action_buttons>
                    <div></div>
                    <div>
                        <button type="button" class="hover-darken" on:click=move |ev| on_undo.call(ev)>
                            <div>Undo</div>
                        </button>
                        <button type="submit" class=stylance::classes!(style::confirm, "hover-darken")>
                            <div>Submit</div>
                        </button>
                    </div>
                </div>
            </ActionForm>
        </div>
    }
}
