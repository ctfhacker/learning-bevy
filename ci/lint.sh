#!/usr/bin/env bash

set -eux -o pipefail

# Ensure the code is formatted properly 
cargo fmt --all -- --check || exit 1

# Bevy lint
bevy lint
