pub mod compress;
pub mod error;
pub mod lzw;
pub mod rle;

use std::env;
use std::io;
use std::io::prelude::*;

type CompressResult = Result<(), error::CompressError>;
type AnyCompress = &'static dyn compress::Compress;

fn main() -> CompressResult {
    let args: Vec<String> = env::args().collect();
    let comp: AnyCompress = if args.contains(&"--rle".to_string()) {
        &rle::RLE
    } else if args.contains(&"--lzw".to_string()) {
        &lzw::LZW
    } else {
        &rle::RLE
    };

    if args.contains(&"-e".to_string()) {
        encode(comp)
    } else if args.contains(&"-d".to_string()) {
        decode(comp)
    } else {
        debug(comp)
    }
}

fn encode(comp: AnyCompress) -> CompressResult {
    let mut data = String::new();
    io::stdin().read_to_string(&mut data)?;
    io::stdout().write_all(&comp.encode(data))?;
    Ok(())
}

fn decode(comp: AnyCompress) -> CompressResult {
    let mut data: Vec<u8> = Vec::new();
    io::stdin().read_to_end(&mut data)?;
    println!("{}", comp.decode(data)?);
    Ok(())
}

fn debug(comp: AnyCompress) -> CompressResult {
    let mut data = String::new();
    io::stdin().read_to_string(&mut data)?;
    let enc = comp.encode(data.clone());
    let dec = comp.decode(enc.clone())?;
    println!("In:  {}\nEnc:  {:?}\nDec:  {}", data, enc, dec);
    Ok(())
}
