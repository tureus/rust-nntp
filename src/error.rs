#[derive(Debug)]
pub enum NNTPError {
    IO(std::io::Error),
    UnexpectedCode(String),
    ReadLineFailed,
    TLSFailed,
    Other,
}

impl From<std::io::Error> for NNTPError {
    fn from(err: std::io::Error) -> Self {
        NNTPError::IO(err)
    }
}
