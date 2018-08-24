#!/bin/bash
rm target/out.stacks; 
sudo dtrace -c './target/release/samvival' -o target/out.stacks -n 'profile-997 /execname == "samvival"/ { @[ustack(100)] = count(); }';
~/Code/third-party-libs/FlameGraph/stackcollapse.pl target/out.stacks | ~/Code/third-party-libs/FlameGraph/flamegraph.pl > target/perf-graph.svg;
open -a "google chrome" target/perf-graph.svg
