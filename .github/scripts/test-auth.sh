#!/bin/bash
set -e

cargo test --release --features hickory-dns,external-account --manifest-path foundation/auth/Cargo.toml
cargo test --release --no-default-features --features native-tls,external-account,jwt-rust-crypto --manifest-path foundation/auth/Cargo.toml
