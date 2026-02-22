#!/bin/bash
set -e

cargo test --release --features default-tls,rustls-tls,trace,auth,external-account,jwt-aws-lc-rs --manifest-path kms/Cargo.toml