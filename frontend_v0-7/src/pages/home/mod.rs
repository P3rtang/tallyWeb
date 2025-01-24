use super::*;

use elements::{InfoBox, Navbar, SelectField, TextField};
use std::collections::HashSet;

mod page;
mod sidebar;
pub use page::HomePage;
use sidebar::SidebarContent;

stylance::import_style!(style, "./home.module.scss");

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Selection {
    slct: HashSet<CountableId>,
}

impl Selection {
    fn new() -> Self {
        Self {
            slct: HashSet::new(),
        }
    }

    pub fn contains(&self, key: &CountableId) -> bool {
        self.slct.contains(key)
    }

    pub fn toggle(&mut self, key: &CountableId) {
        if !self.slct.remove(key) {
            self.slct.insert(*key);
        }
    }
}

impl Params for Selection {
    fn from_map(map: &leptos_router::params::ParamsMap) -> std::result::Result<Self, ParamsError> {
        let selection = map
            .clone()
            .into_iter()
            .filter_map(|p| {
                p.0.starts_with("slct")
                    .then_some(p.0.to_string() + "=" + &p.1)
            })
            .collect::<Vec<_>>()
            .join("&");

        // strip off brackets
        serde_qs::from_str::<Self>(&selection)
            .map_err(|err| ParamsError::Params(std::sync::Arc::new(err)))
    }
}

impl IntoIterator for Selection {
    type Item = CountableId;

    type IntoIter = std::collections::hash_set::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.slct.into_iter()
    }
}
