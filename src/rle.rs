use std::convert::TryInto;
use std::str::Bytes;

struct Run {
    char: u8,
    len: usize,
}

pub fn encode(data: String) -> Vec<u8> {
    runs(data.bytes())
        .iter()
        .flat_map(|x| encode_run(x))
        .collect()
}

pub fn decode(data: Vec<u8>) -> String {
    String::from_utf8(
        data.chunks(9)
            .flat_map(|val| run_to_bytes(decode_run(val)))
            .collect::<Vec<u8>>(),
    )
    .unwrap()
}

fn encode_run(run: &Run) -> Vec<u8> {
    let mut bs = run.len.to_be_bytes().to_vec();
    bs.push(run.char);
    bs
}

fn decode_run(run: &[u8]) -> Run {
    let cnt: [u8; 8] = run[..8].try_into().unwrap();
    let c = run[8];
    Run {
        char: c,
        len: usize::from_be_bytes(cnt),
    }
}

fn run_to_bytes(run: Run) -> Vec<u8> {
    [run.char].repeat(run.len)
}

fn runs(mut data: Bytes) -> Vec<Run> {
    let mut runs: Vec<Run> = Vec::new();
    if let Some(c) = data.next() {
        let mut idx = 0;
        runs.push(Run { char: c, len: 1 });
        for c in data {
            let prev = &runs[idx];
            if prev.char == c {
                runs[idx] = Run {
                    char: c,
                    len: prev.len + 1,
                };
            } else {
                runs.push(Run { char: c, len: 1 });
                idx += 1;
            }
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
            assert_eq!(roundtrip, test.to_string());
        }
    }
}
