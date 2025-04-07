use super::*;

mod header;
mod infobox;

use header::InfoHeader;
pub use infobox::InfoBox;

use crate::hooks::use_history;
use app::UserName;
use elements::icon::*;
use home::Selection;

stylance::import_style!(style, "infobox.module.scss");
