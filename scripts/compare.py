#!/usr/bin/env python
from subprocess import Popen, PIPE
from itertools import product
from pathlib import Path
import sys


def program_to_cmd(program: str, player: str):
    return program.replace("x", player).split(" ")


def run_test(program_a: str, program_b: str, start_moves: list[str]):
    # spawn the programs
    print(f"Running {program_a} vs {program_b}")

    process_a = Popen(
        program_to_cmd(program_a, "x"), stdout=PIPE, stdin=PIPE, stderr=PIPE, text=True
    )
    process_b = Popen(
        program_to_cmd(program_b, "o"), stdout=PIPE, stdin=PIPE, stderr=PIPE, text=True
    )

    end = 0

    moves = 0

    while end == 0:
        moves += 1
        # print("Program A turn")
        while True:
            line_a = process_a.stdout.readline().rstrip()
            if line_a == "$":
                process_b.stdin.write("$\n")
                end = 1
                break

            if not line_a or line_a[0] != "!":
                print(f"Program A: {line_a}")
                continue

            tile = line_a[1:]

            print(f"Program A: {tile}")

            process_b.stdin.write(f"{tile}\n")
            process_b.stdin.flush()
            break

        if end:
            break

        # print("Program B turn")
        moves += 1
        while True:
            line_b = process_b.stdout.readline().rstrip()

            if line_b == "$":
                process_a.stdin.write("$\n")
                end = 2
                break

            if not line_b or line_b[0] != "!":
                print(f"Program B: {line_b}")
                continue

            tile = line_b[1:]

            print(f"Program B: {tile}")

            process_a.stdin.write(f"{tile}\n")
            process_a.stdin.flush()
            break

    print(
        f"{program_a} (A) won" if end == 1 else f"{program_b} (B) won",
        f"in {moves} moves",
    )


assert len(sys.argv) > 1, "Requires at least one program to compare"

played = set()

all_programs = set(sys.argv[1:])

for programs in product(all_programs, repeat=2):
    if programs in played:
        continue

    played.add(programs)
    played.add((programs[1], programs[0]))

    for start_moves in [["h7", "i8"], ["e5", "f6"], ["a1", "b2"]]:
        run_test(*programs, start_moves)
