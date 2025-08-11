use std::ops::Deref;
pub mod trash;

pub struct AsciiString {
    value: String,
}

impl Deref for AsciiString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl TryFrom<&str> for AsciiString {
    type Error = crate::error::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        for char in value.chars() {
            if !char.is_ascii() || char.is_control() {
                return Err(crate::error::Error::NotAscii(char));
            }
        }
        Ok(Self {
            value: value.to_owned(),
        })
    }
}
