[default]
default:
    @just --list

build:
    @cargo build --workspace

clippy:
    @cargo clippy --workspace

fmt:
    @# We fmt with nightly because I like vertical import fmt which is unstable
    @cargo +nightly fmt

run:
    @cargo run -- ./test.basm

dbg:
    @rust-gdb --args cargo run -- ./test.basm

assemble file:
    @cargo run --package assembler --bin assembler -- {{file}}

disassemble file:
    @cargo run --package disassembler --bin disassembler -- {{file}}

test:
    @cargo test --workspace
