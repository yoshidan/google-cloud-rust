#!/bin/bash
set -e

cargo test --release --features trace  --manifest-path bigquery/Cargo.toml