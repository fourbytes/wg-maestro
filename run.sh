#!/bin/sh
set -e

cargo build
sudo setcap CAP_NET_ADMIN=+eip "$(pwd)/target/debug/wg-maestro"
./target/debug/wg-maestro $@
