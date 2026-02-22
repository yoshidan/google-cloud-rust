#!/bin/bash
set -e

cargo test --release --features trace,bytes --manifest-path pubsub/Cargo.toml