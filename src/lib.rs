pub mod entry;
pub mod error;
mod event;
mod parser;
mod reader;

pub use error::VdfError;

pub type Result<T, E = VdfError> = std::result::Result<T, E>;
pub use event::{EntryEvent, Event, GroupEndEvent, GroupStartEvent, Item};
pub use parser::Token;
pub use reader::Reader;
