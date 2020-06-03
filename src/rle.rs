use std::convert::{TryFrom, TryInto};

use crate::error::DecodeError;

struct Run {
    char: u8,
    len: usize,
}

impl Run {
    fn into_bytes(self) -> Vec<u8> {
        [self.char].repeat(self.len)
    }
}

impl TryFrom<&[u8]> for Run {
    type Error = DecodeError;
    fn try_from(bs: &[u8]) -> Result<Self, Self::Error> {
        let cnt: [u8; 8] = bs[..8].try_into().or_else(|_| Err(DecodeError))?;
        let c = bs[8];
        Ok(Run {
            char: c,
            len: usize::from_be_bytes(cnt),
        })
    }
}

impl From<&Run> for Vec<u8> {
    fn from(run: &Run) -> Self {
        let mut bs = run.len.to_be_bytes().to_vec();
        bs.push(run.char);
        bs
    }
}

pub fn encode(data: String) -> Vec<u8> {
    runs(data.bytes()).iter().flat_map(Vec::from).collect()
}

pub fn decode(data: Vec<u8>) -> Result<String, DecodeError> {
    data.chunks(9)
        .map(|val| Run::try_from(val).and_then(|r| Ok(r.into_bytes())))
        .collect::<Result<Vec<_>, _>>()
        .and_then(|bs| {
            String::from_utf8(bs.into_iter().flatten().collect()).or_else(|_| Err(DecodeError))
        })
}

fn runs<T: IntoIterator<Item = u8>>(data: T) -> Vec<Run> {
    let mut runs: Vec<Run> = Vec::new();
    for c in data {
        if let Some(prev) = runs.last_mut() {
            if prev.char == c {
                prev.len += 1;
            } else {
                runs.push(Run { char: c, len: 1 });
            }
        } else {
            runs.push(Run { char: c, len: 1 });
        }
    }
    runs
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
            let roundtrip = decode(encode(test.to_string()));
            assert!(roundtrip.is_ok());
            assert_eq!(roundtrip.unwrap(), test.to_string());
        }
    }
}
