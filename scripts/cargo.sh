#!/bin/sh

cargo build --manifest-path $1/Cargo.toml --release && cp $1/target/release/me-tv $2
