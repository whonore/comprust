pub mod error;
pub mod rle;

use std::env;
use std::io;
use std::io::prelude::*;

fn main() -> Result<(), error::CompressError> {
    let args: Vec<String> = env::args().collect();

    if args.contains(&"-e".to_string()) {
        let mut data = String::new();
        io::stdin().read_to_string(&mut data)?;
        io::stdout().write_all(&rle::encode(data))?;
    } else if args.contains(&"-d".to_string()) {
        let mut data: Vec<u8> = Vec::new();
        io::stdin().read_to_end(&mut data)?;
        println!("{}", rle::decode(data)?);
    } else {
        let mut data = String::new();
        io::stdin().read_to_string(&mut data)?;
        let enc = rle::encode(data.clone());
        let dec = rle::decode(enc.clone())?;
        println!("In:  {}\nEnc:  {:#?}\nDec:  {}", data, enc, dec);
    }
    Ok(())
}
