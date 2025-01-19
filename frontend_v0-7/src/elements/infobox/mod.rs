#![allow(clippy::module_inception)]
use super::*;

mod header;
mod infobox;

use header::InfoHeader;
pub use infobox::InfoBox;

use crate::hooks::{use_referer, RefererOptions};
use app::UserName;
use elements::icon::*;
use home::Selection;
