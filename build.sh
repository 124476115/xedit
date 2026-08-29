#!/bin/bash
set -e

echo "Building Code Editor (Rust)..."
cargo build --release

echo "Binary location: target/release/code-editor"
echo "Copy to desired location or run directly."