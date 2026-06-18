# https://just.systems

[default]
default:
    @just --list

build:
    @cargo build

clippy:
    @cargo clippy

fmt:
    @cargo +nightly fmt

run:
    @cargo run -- ./test.bin
