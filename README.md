# Gomoku engine

## Description

simple gomoku engine written in Rust  
uses AlphaBeta pruning, caching with zobrist hashing, multithreading and iterative deepening

## Modes

### 1. interactive

`gomoku <player> <depth> [start]`

- player - which symbol should engine play as ('x' or 'o')
- depth - how many plies in future should the engine look
- start - should the engine be first player ('true' or 'false')

reads from stdin in format `x,y`

### 2. single position

`gomoku <player> <depth> debug <path-to-input-file>`

- player - which symbol should engine play as ('x' or 'o')
- depth - how many plies in future should the engine look
- path-to-input-file - path to file in specified format

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
