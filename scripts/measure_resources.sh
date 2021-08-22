#!/bin/sh

cargo build --release
/usr/bin/time -f "RSS: %M; Time: %E; CPU: %P" ./target/release/gomoku "$@"