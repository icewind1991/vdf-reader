pub mod entry;
mod error;
mod parser;
mod reader;

pub use error::VdfError;

pub type Result<T, E = VdfError> = std::result::Result<T, E>;
pub use parser::Token;
pub use reader::{Event, Item, Reader};
