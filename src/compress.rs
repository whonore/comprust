use crate::error::DecodeError;

pub trait Compress {
    fn encode(&self, data: String) -> Vec<u8>;
    fn decode(&self, data: Vec<u8>) -> Result<String, DecodeError>;
}
