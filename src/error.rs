/// This crate's errors
#[derive(Debug)]
pub enum Error {
    /// The character is not in the ASCII table.
    NotAscii(char),
    /// The string was not found.
    NotFound(String),
    /// The date could not be parsed.
    #[cfg(feature = "trash")]
    DateParsing(time::error::Parse),
    /// The date could not be formated.
    #[cfg(feature = "trash")]
    DateFormat(time::error::Format),
}
