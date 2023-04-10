#!/bin/sh

set -e

echo "Running test "
echo
cargo test -- --nocapture

echo "Running format "
echo
cargo fmt

echo "Running clippy "
echo
cargo clippy


echo "Ok"

