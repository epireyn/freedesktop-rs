#[derive(Debug)]
pub enum Error {
    NotAscii(char),
    NotFound(String),
    DateParsing(time::error::Parse),
    DateFormat(time::error::Format),
}
