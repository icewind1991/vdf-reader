pub mod entry;
pub mod error;
mod event;
mod lexer;
mod reader;
mod serde;
mod tokenizer;

pub use error::VdfError;

pub type Result<T, E = VdfError> = std::result::Result<T, E>;
pub use crate::serde::{from_entry, from_str};
pub use event::{EntryEvent, Event, GroupEndEvent, GroupStartEvent, Item};
pub use lexer::Token;
pub use reader::Reader;
