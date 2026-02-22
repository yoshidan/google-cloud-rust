#!/bin/bash
set -e

cargo test --release --features default-tls,rustls-tls,hickory-dns,trace,auth,external-account,jwt-aws-lc-rs --manifest-path bigquery/Cargo.toml