# crack :zap:

dynamic instrumentation and symbolic execution-based tools for testing distributed and concurrent implementations

use of ptrace for forcing deterministic execution, combined with [maximal causality reduction](https://parasol.tamu.edu/~jeff/academic/mcr.pdf) for minimizing interesting schedules

## appendix: philosophy of reliable engineering

for large stateful systems, extra specification up-front saves tremendous effort overall

1. TLA+ is useful, but it's costly to learn. we use this to bridge the gap
1. cleanroom methodology: specify the intended behavior of ALL nontrivial blocks
1. simulate asynchronous network conditions by delivering messages {on time, late, never}
   in a test harness that explores different paths (either generative or
   full-state enumeration) depending on testing compute time budget
1. exercise various thread interleavings to draw out violations of invariants

in action:

1. model core communication algos in TLA+ before coding
1. make all messaging pluggable 
1. make all sources of time pluggable
1. rely on typed notions of time to reduce the state space explosion of message
   delivery latencies
1. HEAVILY use `debug_assert!` for all nontrivial blocks
