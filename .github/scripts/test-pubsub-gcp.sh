#!/bin/bash
set -e

cargo test --release --features default-tls,rustls-tls,external-account,trace,bytes,auth,jwt-aws-lc-rs --manifest-path pubsub/Cargo.toml -- --ignored