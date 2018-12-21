#!/bin/sh

cargo build --manifest-path $1/Cargo.toml --release && cp $1/target/release/me-tv $2 && cp $1/target/release/me-tv-record $1/target/release/me-tv-schedule $(dirname $2)
