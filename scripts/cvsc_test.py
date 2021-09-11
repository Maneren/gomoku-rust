import subprocess
from subprocess import PIPE
import time

p1 = subprocess.Popen(('./rust', 'x', '1000', 'true'), stdin=PIPE, stdout=PIPE)
p2 = subprocess.Popen(('python', './gomoku/__init__.py' ), stdin=PIPE, stdout=PIPE)

on_turn = 0
move = '.'
written = False

def write_to_stdin(p, msg):
  p.stdin.write(f'{msg}\n'.encode('utf-8'))
  p.stdin.flush()

def readline(p):
  return p.stdout.readline().decode('utf-8').rstrip()

time.sleep(1)


while p1.poll() is None or p2.poll() is None:
  line = ''
  if on_turn == 0:
    if not written:
      print("\n")
      write_to_stdin(p1, move)
      time.sleep(time_limit)
      write_to_stdin(p1, '')
      written = True

    line = readline(p1)
    print(f"1 {line}")
  else:
    if not written:
      print("\n")
      write_to_stdin(p2, '!'+move)
      written = True
      # time.sleep(time_limit + 2)
      # write_to_stdin(p2, '')

    line = readline(p2)
    print(f"2 {line}")

  if line.startswith('$'):
    print(line)
    break

  if line.startswith('!'):
    move = line
    on_turn = (on_turn + 1) % 2
    written = False

print('Done')
p1.kill()
p2.kill()
