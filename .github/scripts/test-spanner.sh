#!/bin/bash
set -e

cargo test --release --features trace,otel-metrics --manifest-path spanner/Cargo.toml