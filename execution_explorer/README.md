# execution explorer

1. write buggy multithreaded code (`cargo build` in this directory)
1. tell execution_explorer.py what the points of interest are (edit configuration section)
1. `while true; do ./execution_explorer.py; done`
1. wait for the gdb shell to pop on an invariant violation

## how it works

1. waits for all whitelisted threads to reach the rendezvous point
1. inserts breakpoints at interesting locations
1. runs threads in all orders until we've explored all permutations
1. if we reach an invariant location (that should never be hit, like
   a panic) then we drop into a gdb shell and stop saving new state files.
   when the script is re-run with the same state file, it should trigger
   the same bug if you've properly defined the "interesting" set of
   interleaving points.

## quirks

* gdb bugs out for me (`8.0.1`) when I try to restart, which is why we
  rerun gdb each time in a loop. eventually I may write a ptrace
  program that makes all this much faster.
