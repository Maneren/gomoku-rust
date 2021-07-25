#!/bin/bash

cargo build --release
command /usr/bin/time -f "RSS: %M; Time: %E; CPU: %P" ./target/release/gomoku $@