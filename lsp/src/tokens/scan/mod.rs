//! Text scanners shared by the token classifier and the server: UTF-16
//! column conversion and key-path / separator scanning.

mod encoding;
mod key_paths;

pub use encoding::{byte_to_utf16, prefix_by_encoding};
pub(crate) use key_paths::find_key_separator;
pub use key_paths::{cursor_is_after_separator, split_dotted};
