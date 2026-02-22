#!/bin/bash
set -e

cargo test --release --features trace --manifest-path pubsub/Cargo.toml -- --ignored