//! Output formatters
//!
//! Functions for formatting configuration output

mod json;
mod table;

pub use json::format_json;
pub use table::format_table;
