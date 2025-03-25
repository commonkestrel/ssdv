use std::{fs::File, io::{Read, Write}, path::Path, process::ExitCode};

use ssdv::Quality;

const CALLSIGN: &[u8; 6] = b"SOMETH";
const IMAGE_ID: u8 = 1;
const QUALITY: Quality = Quality::Q3;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 3 {
        println!("Usage: ssdv <INPUT> <OUTPUT>");
    }

    let input = Path::new(&args[1]);

    let mut image = Vec::new();
    let mut in_file = File::open(input).expect("Unable to open input file");
    in_file
        .read_to_end(&mut image)
        .expect("Unable to read from file");

    let encoder = ssdv::Encoder::new(*CALLSIGN, IMAGE_ID, QUALITY, image);

    let output = Path::new(&args[2]);
    let mut out_file = File::create(output).expect("Unable to create output file");

    for (i, chunk) in encoder.enumerate() {
        match chunk {
            Ok(c) => out_file
                .write_all(&c)
                .expect("Unable to write to output file"),
            Err(err) => println!("Failed to encode chunk {i}: {err:?}"),
        }
    }

    return ExitCode::SUCCESS;
}
