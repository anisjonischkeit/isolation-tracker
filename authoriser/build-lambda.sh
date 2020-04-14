#! /bin/sh

alias rust-musl-builder='docker run --rm -it -v "$(pwd)":/home/rust/src -v ~/.cargo/registry:/home/rust/.cargo/registry ekidd/rust-musl-builder'
rust-musl-builder cargo build --release
zip -j rust.zip ./target/x86_64-unknown-linux-musl/release/bootstrap
