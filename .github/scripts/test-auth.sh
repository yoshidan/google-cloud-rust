#!/bin/bash
set -e

cargo test --release --features default-tls,rustls-tls,hickory-dns,external-account,jwt-aws-lc-rs --manifest-path foundation/auth/Cargo.toml