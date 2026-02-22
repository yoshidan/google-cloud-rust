#!/bin/bash
set -e

cargo test --release --features trace --manifest-path artifact-registry/Cargo.toml