use super::*;

pub fn use_referer(options: Options) -> Memo<Option<String>> {
    let page_context = use_context::<PageContext>()
        .ok_or(AppError::PageContextUnavailable("use_referer".into()))
        .unwrap();

    let location = use_location();

    if options.is_refering {
        page_context.referer.0.set(Some(format!(
            "{}?{}",
            location.pathname.get_untracked(),
            location.search.get_untracked()
        )))
    }

    Memo::new(move |_| page_context.referer.0.get())
}

#[derive(Debug, Default)]
pub struct Options {
    pub is_refering: bool,
}
