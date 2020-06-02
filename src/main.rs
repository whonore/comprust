pub mod rle;

use std::env;
use std::io;
use std::io::prelude::*;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.contains(&"-e".to_string()) {
        let mut data = String::new();
        io::stdin().read_to_string(&mut data).unwrap();
        io::stdout().write_all(&rle::encode(data)).unwrap();
    } else if args.contains(&"-d".to_string()) {
        let mut data: Vec<u8> = Vec::new();
        io::stdin().read_to_end(&mut data).unwrap();
        println!("{}", rle::decode(data));
    } else {
        let mut data = String::new();
        io::stdin().read_to_string(&mut data).unwrap();
        let enc = rle::encode(data);
        let dec = rle::decode(enc);
        println!("{}", dec);
    }
}
