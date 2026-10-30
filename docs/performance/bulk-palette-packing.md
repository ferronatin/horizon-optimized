# Bulk palette packing

The first optimization in this fork replaces the per-cell packing loop in
`crates/horizon-core/src/paletted.rs::pack_words` with a word-at-a-time loop.
It is used by `from_values`, global-palette construction and `pack` (saving).
It does not change simulation, render distance, graphics settings or features.

## Work removed

The old loop called `write` for every id: locate the padded word, load it,
clear the cell and store it back. The new loop accumulates a word locally and
pushes it once. There is no per-cell location calculation and no initial
zero-fill of the output vector. Allocation size and packed layout stay the same.

The mask operation is deliberately retained: public constructors can supply
out-of-range values, and their old bit-spill behaviour must also stay identical.
There is no unsafe code, CPU-specific instruction set or new dependency.

## Correctness

`bulk_packing_matches_cell_writes_at_every_width` compares every output word
against the original cell writer for all widths 1 through 32. It covers empty,
single-cell, full-word, partial-word, biome and block-section lengths; zero,
maximum fitting, sequential, random, `u32::MAX` and random out-of-range ids.
It also decodes every entry independently and checks unused bits for fitting
ids. Existing tests cover palette growth, random edits, wire round trips,
saved-data round trips, malformed inputs and first-error ordering.

The benchmark checks equality before timing, including full-section `pack`
results (palette order, bits and data). It keeps the original packing loop as a
test-only baseline in the same binary and compiled with the same flags.

## Measurement

2026-10-30, Linux x86_64, AMD EPYC 9V74 virtual machine, rustc 1.99.1,
release profile (thin LTO, one codegen unit), based on upstream `0a341cc353`.
Each case uses 25 rounds of 2,000 calls per implementation, alternating their
order, discarding the first round and reporting the upper median of the other
24. Inputs and outputs pass through `black_box`; allocations and drops count.
The recorded run had no other Rust build or test running. The host is shared,
with no clock or core pinning, so these are local microbenchmarks, not FPS
claims, confidence intervals or guarantees for other computers.

| Operation | Original (us) | Optimized (us) | Speedup |
| --- | ---: | ---: | ---: |
| Pack 4,096 ids, 4 bits | 8.079 | 3.236 | 2.497x |
| Pack 4,096 ids, 5 bits | 6.144 | 3.219 | 1.909x |
| Pack 4,096 ids, 6 bits | 5.573 | 3.229 | 1.726x |
| Pack 4,096 ids, 7 bits | 6.212 | 3.957 | 1.570x |
| Pack 4,096 ids, 8 bits | 5.657 | 3.398 | 1.665x |
| Pack 4,096 ids, 12 bits | 5.298 | 3.639 | 1.456x |
| Pack 4,096 ids, 16 bits | 5.430 | 3.943 | 1.377x |
| Complete section pack, terrain | 15.346 | 10.875 | 1.411x |
| Complete section pack, surface | 16.839 | 13.985 | 1.204x |
| Complete section pack, global | 16.887 | 15.586 | 1.083x |
| Complete section pack, large ids | 60.622 | 57.320 | 1.058x |
| Complete biome pack | 0.331 | 0.273 | 1.210x |
| Complete section pack, single value | 4.438 | 4.329 | 1.025x |
| Complete section pack, 17 values | 17.020 | 14.104 | 1.207x |

The single-value path does not call the changed function; its small difference
is noise. Complete-section results include unpacking and palette construction,
which explains the smaller gain than in the packing kernel alone. Large ids
remain dominated by palette construction. Raw medians and setup are recorded in
`benchmarks/bulk-palette-packing-2026-10-30.json`.

## Reproduce

```sh
cargo test -p horizon-core --lib --bins --tests --locked
cargo test -p horizon-core --doc --locked
cargo clippy -p horizon-core --all-targets --locked -- -D warnings
cargo fmt --all --check
cargo test -p horizon-core --lib --release --locked benchmark_bulk_packing -- --ignored --nocapture
```

The core suite passes 119 tests, with three microbenchmarks ignored by default.
The crate's Clippy, formatting and doc tests pass. After providing pkg-config
and ALSA/udev development files through a temporary local sysroot, both full
workspace all-target Clippy and the no-default-features server Clippy pass
with warnings denied, as run by `pr_ready --quick`.

The overall self-check remains nonzero: one of 480 Python tests cannot read
its own process RSS through `ps` in this environment. The other 479 pass.
The same test also fails in an untouched checkout of `0a341cc353`
(`test_perf_bench.py`: 14 pass, the same one errors). No test was disabled
or changed to hide that limitation.

Vanilla differential harnesses and an in-game FPS comparison require the
player's prepared Minecraft files, absent in this environment. They have not
been run; no whole-game performance or complete parity claim is made here.

## Next measurements

Keep this change isolated while validating world loading/saving on real game
data. Then profile frame-time spikes during exploration, worker contention,
mesh uploads and repeated block/collision lookups with the same world, settings
and route. Each subsequent change should have its own before/after evidence
and behaviour comparison. See `docs/benchmark.md` and `docs/parity-workflow.md`.
