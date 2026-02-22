#!/bin/bash
set -e

cargo test --release --features auth,default-tls,external-account,hickory-dns,rustls-tls,jwt-aws-lc-rs,trace --manifest-path storage/Cargo.toml