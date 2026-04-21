#!/usr/bin/env bash
set -e
echo "  Building fledge-plugin-todo (Rust)..."
cargo build --release --quiet
echo "  Build complete."
