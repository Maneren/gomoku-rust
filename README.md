# Gomoku engine

## Description

Simple gomoku engine written in Rust with CLI and [GUI](https://github.com/Maneren/gomoku-gui-dioxus).

## Installation

Requires [Rust](https://www.rust-lang.org/tools/install) installed.

```sh
cargo install --path .
```

## Usage

### Interactive

`gomoku <player> <time>`

- player - who should go first - engine always plays as `x`, player as `o`
- time - time limit for computing in milliseconds

Reads input from `stdin` in format `d6` (letter is horizontal, number is vertical)

### Single position evaluation

`gomoku <player> <time> -d <file>`

- player - which symbol should engine evaluate as (`x` or `o`)
- time - time limit for searching in milliseconds
- file - path to file with position to evaluate

Evaluates single positions and returns the best move.

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

Available here: [Gomoku GUI](https://github.com/Maneren/gomoku-gui-dioxus).
