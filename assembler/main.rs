use std::{fs::File, io::{BufRead, BufReader, Write}, path::PathBuf, str::FromStr};

use common::ops::{asm, bytecode::{MachineCode, ToMachineCode}};


fn main() {
    let path = PathBuf::from_str(&std::env::args().nth(1).expect("Please provide path to assembly file as first argument")).expect("Failed to parse argument as a path");

    let reader = BufReader::new(File::open(path).expect("Could not open assembly file"));

    let mut assembled = MachineCode::new();

    for op in reader.lines().filter_map(|line| {
        let line = line.expect("Failed to parse line of file");
        if line.is_empty() || line.starts_with("//") {
            None
        } else {
            Some(asm::Operation::try_from(line.as_str()).expect("Invalid operation encountered"))
        }
    }) {
        assembled.concat(op.assemble());
    }

    let mut file = File::create("prog.sram").expect("Cannot create prog.sram file!");
    file.write_all(&assembled).expect("Failed to write program to file");
}
