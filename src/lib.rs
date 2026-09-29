//! Independent HTML, CSS, script, layout, and raster implementation.
#![forbid(unsafe_code)]
pub mod css;
pub(crate) mod cssom;
pub mod date_host;
pub(crate) mod document_url;
pub mod dom;
pub mod graphics;
mod image_limits;
mod js_date;
mod js_identifier;
pub mod js_string;
mod js_uri;
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
