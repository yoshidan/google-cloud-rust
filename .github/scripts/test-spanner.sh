#!/bin/bash
set -e

cargo test --release --features trace,auth,default-tls,rustls-tls,external-account,otel-metrics,jwt-aws-lc-rs --manifest-path spanner/Cargo.toml