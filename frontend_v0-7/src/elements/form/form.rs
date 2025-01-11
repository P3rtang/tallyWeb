use serde::de::DeserializeOwned;
use server_fn::{client::Client, codec::PostUrl, request::ClientReq, ServerFn};
use web_sys::FormData;

use super::*;

stylance::import_style!(style, "./form.module.scss");

#[component]
pub fn Form<ServFn>(action: ServerAction<ServFn>, children: ChildrenFn) -> impl IntoView
where
    ServFn: DeserializeOwned + ServerFn<InputEncoding = PostUrl> + Clone + Send + Sync + 'static,
    <<ServFn::Client as Client<ServFn::Error>>::Request as ClientReq<ServFn::Error>>::FormData:
        From<FormData>,
    ServFn::Output: Send + Sync + 'static,
    ServFn::Error: Send + Sync + 'static,
{
    view! {
        <ActionForm action>
            <div class=style::form>
                {children()}
            </div>
        </ActionForm>
    }
}
