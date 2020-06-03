#[derive(Debug)]
pub struct EncodeError;

#[derive(Debug)]
pub struct DecodeError;

#[derive(Debug)]
pub enum CompressError {
    EncodeError(EncodeError),
    DecodeError(DecodeError),
    IOError(std::io::Error),
}

impl From<EncodeError> for CompressError {
    fn from(err: EncodeError) -> Self {
        CompressError::EncodeError(err)
    }
}

impl From<DecodeError> for CompressError {
    fn from(err: DecodeError) -> Self {
        CompressError::DecodeError(err)
    }
}

impl From<std::io::Error> for CompressError {
    fn from(err: std::io::Error) -> Self {
        CompressError::IOError(err)
    }
}
