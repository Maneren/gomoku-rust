# Gomoku engine

## Description

simple gomoku engine written in Rust  
uses AlphaBeta pruning, caching with zobrist hashing, multithreading and iterative deepening

## Modes

### 1. interactive

`gomoku <player> <time> [start]`

- player - which symbol should engine play as ('x' or 'o')
- time - time limit for searching in milliseconds
- start - should the engine be play first ('true' or 'false')

reads from stdin in format `d6` (letter is horizontal, number is vertical)

### 2. single position

`gomoku <player> <time> debug <input-file>`

- player - which symbol should engine play as ('x' or 'o')
- time - time limit for searching in milliseconds
- input-file - path to file in specified format

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
