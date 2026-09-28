//! Independent HTML, CSS, script, layout, and raster implementation.
#![forbid(unsafe_code)]
pub mod css;
pub(crate) mod document_url;
pub mod dom;
pub mod graphics;
mod image_limits;
pub mod js_string;
pub mod layout;
pub mod net;
pub mod page;
mod regexp;
pub mod script;
mod selectors;
mod stylesheet_loading;
pub mod svg;
mod text_encoding;
pub mod worker;
