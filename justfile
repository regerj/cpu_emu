# https://just.systems

[default]
default:
    @just --list

build:
    @cargo build

clippy:
    @cargo clippy

fmt:
    @# We fmt with nightly because I like vertical import fmt which is unstable
    @cargo +nightly fmt

run:
    @cargo run -- ./test.basm

dbg:
    @rust-gdb --args cargo run -- ./test.basm

test:
    @cargo test
