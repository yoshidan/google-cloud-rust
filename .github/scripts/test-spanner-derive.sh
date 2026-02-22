#!/bin/bash
set -e

cargo test --release --all-features --manifest-path spanner-derive/Cargo.toml