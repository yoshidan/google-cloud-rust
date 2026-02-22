#!/bin/bash
set -e

cargo test --release --features trace --manifest-path kms/Cargo.toml