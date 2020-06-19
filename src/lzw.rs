use std::collections::HashMap;
use std::fmt;
use std::iter::Iterator;
use std::ops::{AddAssign, Index};

use crate::compress::Compress;
use crate::error::DecodeError;

struct Bits {
    bits: Vec<(usize, usize)>,
}

impl fmt::Binary for Bits {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.bits
            .iter()
            .map(|(val, width)| write!(f, "{:0width$b}", val, width = width))
            .collect()
    }
}

impl AddAssign<(usize, usize)> for Bits {
    fn add_assign(&mut self, other: (usize, usize)) {
        self.bits.push(other)
    }
}

impl From<Bits> for Vec<u8> {
    fn from(bits: Bits) -> Self {
        format!("{:b}", bits)
            .as_bytes()
            .chunks(8)
            .map(|bs| u8::from_str_radix(std::str::from_utf8(bs).unwrap(), 2).unwrap())
            .collect()
    }
}

struct LZWDict {
    dict: HashMap<Vec<u8>, usize>,
    max_code: usize,
}

impl LZWDict {
    fn new(alph: Vec<Vec<u8>>) -> Self {
        Self {
            max_code: alph.len(),
            dict: alph.iter().zip(1..).map(|(x, y)| (x.clone(), y)).collect(),
        }
    }

    fn nbits(&self) -> usize {
        f32::ceil(f32::log2((self.max_code + 1) as f32)) as usize
    }

    fn insert(&mut self, bs: Vec<u8>) {
        self.max_code += 1;
        self.dict.insert(bs, self.max_code);
    }

    fn contains_key(&self, bs: &[u8]) -> bool {
        self.dict.contains_key(bs)
    }
}

impl Index<&Vec<u8>> for LZWDict {
    type Output = usize;

    fn index(&self, key: &Vec<u8>) -> &Self::Output {
        &self.dict[key]
    }
}

pub struct LZW;

impl Compress for LZW {
    fn encode(&self, data: String) -> Vec<u8> {
        let alph: Vec<Vec<u8>> = (0..=255).map(|b| vec![b]).collect();
        Vec::from(bits(data.bytes(), LZWDict::new(alph)))
    }

    fn decode(&self, data: Vec<u8>) -> Result<String, DecodeError> {
        Ok("".to_string())
    }
}

fn bits<T: IntoIterator<Item = u8>>(data: T, mut dict: LZWDict) -> Bits {
    let mut word = vec![];
    let mut bits = Bits { bits: vec![] };
    for c in data {
        word.push(c);
        if !dict.contains_key(&word) {
            let nbits = dict.nbits();
            dict.insert(word.clone());
            word.pop();
            bits += (dict[&word], nbits);
            word = vec![c];
        }
    }
    if !word.is_empty() {
        bits += (dict[&word], dict.nbits());
    }
    bits += (0, dict.nbits());
    bits
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
            let roundtrip = LZW.decode(LZW.encode(test.to_string()));
            assert!(roundtrip.is_ok());
            assert_eq!(roundtrip.unwrap(), test.to_string());
        }
    }

    #[test]
    fn bits_fmt() {
        assert_eq!(
            format!(
                "{:b}",
                Bits {
                    bits: vec![(1, 2), (3, 5)]
                }
            ),
            "0100011"
        );
        assert_eq!(
            format!(
                "{:b}",
                Bits {
                    bits: vec![(1, 2), (3, 5), (7, 3)]
                }
            ),
            "0100011111"
        );
    }

    #[test]
    fn bits_to_bytes() {
        let bits = Bits {
            bits: vec![(1, 2), (3, 5)],
        };
        assert_eq!(Vec::from(bits), vec![35]);
        let bits = Bits {
            bits: vec![(1, 2), (3, 5), (7, 3)],
        };
        assert_eq!(Vec::from(bits), vec![71, 3]);
    }
}
