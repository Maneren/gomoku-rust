# Gomoku engine

## Description

simple gomoku engine written in Rust with CLI

uses multithreading and iterative deepening

## Modes

### 1. interactive

`gomoku <player> <time>`

- player - who should go first - engine is 'x', player is 'o'
- time - time limit for computing in milliseconds

reads input from stdin in format `d6` (letter is horizontal, number is vertical)

### 2. single position

`gomoku <player> <time> -d <file>`

- player - which symbol should engine evaluate as ('x' or 'o')
- time - time limit for searching in milliseconds
- file - path to file with position to evaluate

evaluates single positions and returns its move

input file example:

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
