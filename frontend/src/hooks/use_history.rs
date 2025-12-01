use leptos_router::{NavigateOptions, hooks::use_navigate, location::Location};

use super::*;

pub trait Navigate {
    fn navigate(self, options: NavigateOptions);
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Url {
    url: String,
}

impl std::fmt::Display for Url {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.url)
    }
}

impl Navigate for Memo<Option<Url>> {
    fn navigate(self, options: NavigateOptions) {
        let navigate = use_navigate();

        if let Some(Url { url }) = self.get_untracked() {
            navigate(&url, options)
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct History {
    history: RwSignal<Vec<Url>>,
}

impl History {
    pub fn back(self) -> Memo<Option<Url>> {
        Memo::new(move |_| {
            let mut history = self.history.get();
            history.pop()
        })
    }

    pub fn save_location(self) {
        let loc = use_location();
        let mut url = loc.pathname.get();

        if !loc.search.get().is_empty() {
            url.push('?');
            url.push_str(&loc.search.get())
        }

        if !loc.hash.get().is_empty() {
            url.push('#');
            url.push_str(&loc.hash.get().to_string());
        }

        self.history.update(|h| h.push(Url { url }));
    }
}

pub fn use_history() -> History {
    let page_context = use_context::<PageContext>()
        .expect("To use the history hook a page context has to provided in a parent component");
    page_context.history
}
