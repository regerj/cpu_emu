//! Two-pass assembler for our toy assembly language.
//!
//! First pass will track program location (offset into binary) in order to track labels.
//!
//! Second pass will perform assembly and replace labels with relative addresses.

use std::{
    fs::File, io::{
        BufRead, BufReader, Read, Write
    }, path::PathBuf, str::FromStr
};

use common::isa;

use crate::{lexer::tokenize, parser::parse};

mod lexer;
mod parser;

fn main() {
    let path = PathBuf::from_str(
        &std::env::args()
            .nth(1)
            .expect("Please provide path to assembly file as first argument"),
    )
    .expect("Failed to parse argument as a path");

    let file = File::open(path).expect("Could not open assembly file");
    let mut reader = BufReader::new(&file);

    let mut s = String::new();
    reader.read_to_string(&mut s).expect("Failed to read input file to string");
    let tokens = tokenize(s).unwrap();
    println!("{tokens:#?}");

    let ops = parse(tokens);
    println!("{ops:#?}");

    let assembled: Vec<_> = BufReader::new(file)
        .lines()
        .filter_map(|line| {
            let line = line.expect("Failed to parse line of file");
            if line.is_empty() || line.starts_with("//") {
                None
            } else {
                Some(
                    isa::Operation::try_from(line.as_str()).expect("Invalid operation encountered"),
                )
            }
        })
        .flat_map(isa::Operation::compile)
        .collect();

    let mut file = File::create("prog.sram").expect("Cannot create prog.sram file!");
    file.write_all(&assembled)
        .expect("Failed to write program to file");
}
