use std::{
    fs::File,
    io::{
        BufReader,
        Read,
    },
    path::PathBuf,
    str::FromStr,
};

use common::isa;

fn main() {
    let path = PathBuf::from_str(
        &std::env::args()
            .nth(1)
            .expect("Please provide path to assembly file as first argument"),
    )
    .expect("Failed to parse argument as a path");

    let mut reader = BufReader::new(File::open(path).expect("Could not open binary file"));

    let mut buf = Vec::new();
    reader.read_to_end(&mut buf).expect("Could not read bytes");

    let mut bytes = buf.iter();

    while let Some(op) = isa::Operation::consume(&mut bytes).expect("Invalid binary") {
        println!("{op}");
    }
}
