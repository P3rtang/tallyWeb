use serde::de::DeserializeOwned;
use server_fn::{client::Client, codec::PostUrl, request::ClientReq, ServerFn};
use web_sys::FormData;

use super::*;

stylance::import_style!(style, "./form.module.scss");

#[component]
pub fn Form<ServFn>(
    action: ServerAction<ServFn>,
    children: ChildrenFn,
    #[prop(into, optional)] title: Option<Signal<String>>,
    #[prop(into, optional)] close_href: Option<Signal<String>>,
    #[prop(into, optional)] on_undo: EventCallback<ev::click, ev::MouseEvent>,
) -> impl IntoView
where
    ServFn: DeserializeOwned + ServerFn<InputEncoding = PostUrl> + Clone + Send + Sync + 'static,
    <<ServFn::Client as Client<ServFn::Error>>::Request as ClientReq<ServFn::Error>>::FormData:
        From<FormData>,
    ServFn::Output: Send + Sync + 'static,
    ServFn::Error: Send + Sync + 'static,
{
    view! {
        <div class=style::header>
            <span>{ title }</span>
            <Show when=move || close_href.is_some()>
                <button class="hover-darken icon">
                    <a href=close_href>
                        <img height="28px" width="28px" src="/icons/tallyweb-cross-white.svg" />
                    </a>
                </button>
            </Show>
        </div>
        <ActionForm action>
            <div class=style::form>
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
    }
}
