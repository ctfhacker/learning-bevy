#!/usr/bin/env bash

# Run normal tests
RUST_BACKTRACE=1 cargo test --all-features
