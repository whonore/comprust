use std::convert::{TryFrom, TryInto};
use std::iter::{FromIterator, Peekable};

use crate::compress::Compress;
use crate::error::DecodeError;

#[derive(Debug)]
struct Run {
    val: u8,
    len: usize,
}

struct Runs<D: Iterator<Item = u8>> {
    data: Peekable<D>,
    len: usize,
}

fn runs<D: IntoIterator<Item = u8>>(data: D) -> Runs<D::IntoIter> {
    Runs {
        data: data.into_iter().peekable(),
        len: 1,
    }
}

impl<I: Iterator<Item = u8>> Iterator for Runs<I> {
    type Item = Run;

    fn next(&mut self) -> Option<Run> {
        while let Some(val) = self.data.next() {
            match self.data.peek() {
                Some(next) if val == *next => {
                    self.len += 1;
                }
                _ => {
                    let run = Run { val, len: self.len };
                    self.len = 1;
                    return Some(run);
                }
            }
        }
        None
    }
}

impl Run {
    fn into_bytes(self) -> Vec<u8> {
        [self.val].repeat(self.len)
    }
}

impl TryFrom<&[u8]> for Run {
    type Error = DecodeError;

    fn try_from(bs: &[u8]) -> Result<Self, Self::Error> {
        let cnt: [u8; 8] = bs[..8].try_into().or_else(|_| Err(DecodeError))?;
        let c = bs[8];
        Ok(Self {
            val: c,
            len: usize::from_be_bytes(cnt),
        })
    }
}

impl From<Run> for Vec<u8> {
    fn from(run: Run) -> Self {
        let mut bs = run.len.to_be_bytes().to_vec();
        bs.push(run.val);
        bs
    }
}

impl FromIterator<Run> for Vec<u8> {
    fn from_iter<I: IntoIterator<Item = Run>>(runs: I) -> Self {
        runs.into_iter().flat_map(Vec::from).collect()
    }
}

pub struct RLE;

impl Compress for RLE {
    fn encode(&self, data: String) -> Vec<u8> {
        runs(data.bytes()).collect()
    }

    fn decode(&self, data: Vec<u8>) -> Result<String, DecodeError> {
        data.chunks(9)
            .map(|val| Run::try_from(val).and_then(|r| Ok(r.into_bytes())))
            .collect::<Result<Vec<_>, _>>()
            .and_then(|bs| {
                String::from_utf8(bs.into_iter().flatten().collect()).or_else(|_| Err(DecodeError))
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode() {
        let tests = vec![
            "".to_string(),
            "abc".to_string(),
            "aabccd".to_string(),
            "aab0bb0012".to_string(),
            "λaé".to_string(),
            "a".repeat(1000),
        ];
        for test in tests.iter() {
            let roundtrip = RLE.decode(RLE.encode(test.to_string()));
            assert!(roundtrip.is_ok());
            assert_eq!(roundtrip.unwrap(), test.to_string());
        }
    }
}
