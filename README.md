# Gomoku engine

## Description

Simple [Gomoku](https://en.wikipedia.org/wiki/Gomoku) engine written in Rust with
CLI and optional [GUI](https://github.com/Maneren/gomoku-gui-dioxus).

## Features

- minimax search
- heavily parallelized using [`rayon`](https://crates.io/crates/rayon)
- iterative deepening with time limit
- 100% safe Rust
- CLI and GUI

## Installation

Requires [Rust toolchain](https://www.rust-lang.org/tools/install) installed
and set-up.

```sh
cargo install --path .
```

Alternatively, the GUI has precompiled binaries for download in
[latest release](https://github.com/Maneren/gomoku-gui-dioxus/releases/latest).

## Usage

### Interactive

`gomoku <player> <time>`

- player - who should go first (`x` or `o`) - engine always plays as `x`,
  human as `o`
- time - time limit for computing in milliseconds

Reads input from `stdin` in format `d6` (letter is horizontal, number is vertical)

### Single position evaluation

`gomoku <player> <time> -d <file>`

- player - which symbol should engine evaluate as (`x` or `o`)
- time - time limit for searching in milliseconds
- file - path to file with position to evaluate

Evaluates the position and prints the best move + the board and some stats.

Input file example:

```txt
---------
---------
---x-----
---xoo---
----xo---
---xxxo--
------oo-
--------x
---------
```

### GUI

More info here: [Gomoku GUI](https://github.com/Maneren/gomoku-gui-dioxus).
