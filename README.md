# peer :globe_with_meridians:

quickly build and test reliable distributed systems

inspiration from:

* erlang OTP
* FoundationDB's dev process
* riak's dev process
* voltdb's dev process
* NASA's Apollo engineering process

## primitives

- [ ] broadcast
- [ ] partition simulation
- [ ] consensus
- [ ] membership
- [ ] liveness
- [ ] gossip

## appendix: philosophy of reliable engineering

for large stateful systems, extra specification up-front saves tremendous effort overall

1. use TLA+ to model whether core algorithms satisfy desired safety properties
1. cleanroom methodology: specify the intended behavior of ALL nontrivial blocks
1. simulate asynchronous network conditions by delivering messages {on time, late, never}
   in a test harness that explores different paths (either generative or
   full-state enumeration) depending on testing compute time budget

in action:

1. model core communication algos in TLA+ before coding
1. make all messaging pluggable 
1. make all sources of time pluggable
1. rely on typed notions of time to reduce the state space explosion of message
   delivery latencies
1. HEAVILY use `debug_assert!` for all nontrivial blocks

