use std::{fs::File, io::{BufRead, BufReader}, path::PathBuf, str::FromStr};

use common::ops::Operation;

fn main() {
    let path = PathBuf::from_str(&std::env::args().nth(1).expect("Please provide path to assembly file as first argument")).expect("Failed to parse argument as a path");

    let reader = BufReader::new(File::open(path).expect("Could not open assembly file"));

    for line in reader.lines().filter_map(|line| {
        let line = line.expect("Failed to parse line of file");
        if line.is_empty() || line.starts_with("//") {
            None
        } else {
            Some(Operation::try_from(line.as_str()).expect("Invalid operation encountered"))
        }
    }) {
    }
}
