#!/bin/sh

cargo build --release

/usr/lib/linux-tools/5.4.0-81-generic/perf record  -m 10000 --call-graph=dwarf ./target/release/gomoku "$@"
/usr/lib/linux-tools/5.4.0-81-generic/perf report --hierarchy -M intel