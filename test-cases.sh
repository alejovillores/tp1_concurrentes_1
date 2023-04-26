#!/bin/sh

set -e
set +v

cargo run -- res/orders.test1.json 2 
cargo run -- res/orders.test2.json 2 
 

echo "OK"