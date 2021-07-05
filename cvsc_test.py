import subprocess
from subprocess import PIPE

p1 = subprocess.Popen(('./target/release/gomoku', 'x', '4', 'true'), stdin=PIPE, stdout=PIPE)
p2 = subprocess.Popen(('./target/release/gomoku', 'o', '4' ), stdin=PIPE, stdout=PIPE)

on_turn = 0
move = ''
written = False

def write_to_stdin(p, msg):
  p.stdin.write(f'{msg}\n'.encode('utf-8'))
  p.stdin.flush()

def readline(p):
  return p.stdout.readline().decode('utf-8').rstrip()

while p1.poll() is None and p2.poll() is None:
  line = ''
  if on_turn == 0:
    if not written:
      print("\n")
      write_to_stdin(p1, move)
      written = True
    
    line = readline(p1)
    print(f"1 {line}")
  else:
    if not written:
      print("\n")
      write_to_stdin(p2, move)
      written = True

    line = readline(p2)
    print(f"2 {line}")

  if line.startswith('$'):
    break

  if line.startswith('!'):
    move = line[1:]
    on_turn = (on_turn + 1) % 2
    written = False

print('Done')
p1.kill()
p2.kill()