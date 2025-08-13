
use std::{error, fmt};

#[derive(Debug)]
pub enum Error {
    Reason(String),
}

impl Error {
    pub fn reason(str: &str) -> Self {
        Self::Reason(str.to_string())
    }
}

impl<T> Into<Result<T, Error>> for Error {
    fn into(self) -> Result<T, Error> {
        Err(self)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl error::Error for Error {}

