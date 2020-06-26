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

struct LZWRevDict {
    dict: HashMap<usize, Vec<u8>>,
}

struct LZWDict {
    dict: HashMap<Vec<u8>, usize>,
    reversed: LZWRevDict,
    max_code: usize,
}

impl LZWDict {
    fn new(alph: Vec<Vec<u8>>) -> Self {
        Self {
            max_code: alph.len(),
            dict: alph.iter().zip(1..).map(|(x, y)| (x.clone(), y)).collect(),
            reversed: LZWRevDict {
                dict: (1..).zip(alph).map(|(x, y)| (x, y)).collect(),
            },
        }
    }

    fn _nbits(n: usize) -> usize {
        f32::ceil(f32::log2((n + 1) as f32)) as usize
    }

    fn nbits(&self) -> usize {
        LZWDict::_nbits(self.max_code + 1)
    }

    fn nbits_next(&self) -> usize {
        LZWDict::_nbits(self.max_code + 1)
    }

    fn insert(&mut self, bs: Vec<u8>) {
        self.max_code += 1;
        debug_assert!(!self.dict.contains_key(&bs));
        self.dict.insert(bs, self.max_code);
    }

    fn rinsert(&mut self, bs: Vec<u8>) {
        self.max_code += 1;
        debug_assert!(!self.reversed.dict.contains_key(&self.max_code));
        self.reversed.dict.insert(self.max_code, bs);
    }

    fn contains_key(&self, bs: &[u8]) -> bool {
        self.dict.contains_key(bs)
    }
}

impl LZWRevDict {
    #[allow(clippy::trivially_copy_pass_by_ref)]
    fn contains_key(&self, code: &usize) -> bool {
        self.dict.contains_key(code)
    }
}

impl Index<&Vec<u8>> for LZWDict {
    type Output = usize;

    fn index(&self, key: &Vec<u8>) -> &Self::Output {
        &self.dict[key]
    }
}

impl Index<usize> for LZWRevDict {
    type Output = Vec<u8>;

    fn index(&self, key: usize) -> &Self::Output {
        &self.dict[&key]
    }
}

pub struct LZW;

impl LZW {
    fn alph() -> Vec<Vec<u8>> {
        (0..=255).map(|b| vec![b]).collect()
    }
}

impl Compress for LZW {
    fn encode(&self, data: String) -> Vec<u8> {
        Vec::from(bits(data.bytes(), LZWDict::new(LZW::alph())))
    }

    fn decode(&self, data: Vec<u8>) -> Result<String, DecodeError> {
        let mut dict = LZWDict::new(LZW::alph());
        let mut bin: String = data.iter().map(|b| format!("{:08b}", b)).collect();
        eprintln!("In:\n{}", bin);

        let mut nbits = dict.nbits();
        let prev_bs = &mut Vec::new();
        let mut out = Vec::new();
        while !bin.is_empty() {
            // split_off
            let (code, bin2) = bin.split_at(nbits);
            let code = usize::from_str_radix(code, 2).unwrap();
            if code == 0 {
                break;
            }

            let mut bs;
            if dict.reversed.contains_key(&code) {
                bs = dict.reversed[code].clone();
            } else {
                debug_assert_eq!(code, dict.max_code + 1);
                bs = prev_bs.clone();
                bs.push(prev_bs[0]);
            };
            eprint!("Emitting {:?} ({})", bs, code);
            out.push(bs.clone());

            if !prev_bs.is_empty() {
                prev_bs.push(bs[0]);
                debug_assert!(!dict.contains_key(&prev_bs));
                eprint!("\t--\tInserting {:?} ({})", prev_bs, dict.max_code);
                dict.rinsert(prev_bs.to_vec());
                nbits = dict.nbits_next();
            }

            eprintln!();
            debug_assert_eq!(bs, dict.reversed[code]);
            *prev_bs = bs;
            bin = bin2.to_string();
        }

        String::from_utf8(out.into_iter().flatten().collect()).or_else(|_| Err(DecodeError))
    }
}

fn bits<T: IntoIterator<Item = u8>>(data: T, mut dict: LZWDict) -> Bits {
    let mut word = vec![];
    let mut bits = Bits { bits: vec![] };
    for c in data {
        word.push(c);
        if !dict.contains_key(&word) {
            eprintln!(
                "Emitting {:?} ({})\t--\tInserting {:?} ({})",
                word[..word.len() - 1].to_vec(),
                dict[&word[..word.len() - 1].to_vec()],
                word,
                dict.max_code
            );
            let nbits = dict.nbits();
            dict.insert(word.clone());
            word.pop();
            bits += (dict[&word], nbits);
            word = vec![c];
        }
    }
    if !word.is_empty() {
        eprintln!("Emitting {:?} ({})", word, dict[&word]);
        bits += (dict[&word], dict.nbits());
    }
    bits += (0, dict.nbits());
    eprintln!("Out:\n{:b}", bits);
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
