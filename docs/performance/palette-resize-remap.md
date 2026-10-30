# Palette growth: resolve each old id once

Stage 2 is based on `e2d7b0bdf0` (stage 1, bulk packing). It changes only
`PalettedContainer::resize` in `crates/horizon-core/src/paletted.rs`.
The comparison below measures the additional change, not both stages combined.

When a local palette fills, resizing walks the section in cell order and
rebuilds the palette. The old implementation remembered only the last id. A
mixed section repeatedly searched the growing new palette for the same values.

The new local-to-local path keeps a 256-entry array of u16 ids on the stack
(512 temporary bytes). Each old id is resolved on its first occurrence and
reused afterwards. No heap allocation or persistent cache is added. The
sentinel is an impossible local palette id, not a block value: registry values
may use all u32 bits. Single-value and global transitions keep the original
algorithm. No settings, simulation rules, RNG calls or features are removed.

The mapping is deliberately lazy. Mapping palette entries in palette-index
order would change vanilla's first-occurrence order and preserve stale entries
that the original resize discards.

## Correctness

The new differential test keeps the entire previous resize implementation as
a test-only reference. It compares the returned id, configuration, palette
order, every packed word and the saved representation. Cases include:

- Every block and biome growth threshold, including global transitions.
- Cyclic ids, deterministic random ids, 128-cell runs and unused palette entries.
- Duplicate palette values, nonzero input padding, existing/new final values
  and u32::MAX as an ordinary value.
- Initial single-value containers and the existing random-edit/round-trip tests.

## Reproduce

```sh
cargo test -p horizon-core --lib --bins --tests --locked
cargo test -p horizon-core --doc --locked
cargo clippy -p horizon-core --all-targets --locked -- -D warnings
cargo fmt --all --check
cargo test -p horizon-core --lib --release --locked benchmark_palette_resize -- --ignored --nocapture
```

The stage 2 self-check passes formatting, workspace all-target Clippy, the
no-default-features server Clippy (both with warnings denied), 120 Rust tests,
doc tests and the public-tree check. Four benchmarks are ignored by default.
The only overall `pr_ready --quick` failure is the same pre-existing Python RSS
test recorded in stage 1: 479 of 480 Python tests pass; the remaining test cannot
read its own process through `ps` here and also fails in the untouched base.
Native development dependencies were provided through a temporary local sysroot.

## Results

2026-10-30, AMD EPYC 9V74 shared Linux x86_64 VM, rustc 1.99.1,
release profile with thin LTO and one codegen unit. Each case alternates old/new
order over 25 rounds of 1,000 calls, discards the first round and reports the
upper median of the remaining 24. Both sides include cloning the same input,
resizing and dropping the output. Equality is checked before timing. The
recorded run had no other Rust build/test running; CPU clocks/cores were not
pinned. Fixtures are synthetic, with registry values fitting their global
widths. Raw medians for all 32 cases are in
`benchmarks/palette-resize-remap-2026-10-30.json`.

| Resize case | Original (us) | Optimized (us) | Speedup |
| --- | ---: | ---: | ---: |
| blocks, 4 to 5 bits, random | 30.095 | 14.165 | 2.125x |
| blocks, 4 to 5 bits, 128-cell runs | 12.385 | 10.998 | 1.126x |
| blocks, 5 to 6 bits, random | 71.650 | 16.136 | 4.440x |
| blocks, 5 to 6 bits, 128-cell runs | 14.845 | 13.008 | 1.141x |
| blocks, 6 to 7 bits, random | 105.108 | 17.069 | 6.158x |
| blocks, 6 to 7 bits, 128-cell runs | 15.814 | 13.495 | 1.172x |
| blocks, 7 to 8 bits, random | 145.952 | 17.601 | 8.292x |
| blocks, 7 to 8 bits, 128-cell runs | 12.349 | 12.011 | 1.028x |
| blocks, 8 to 16 bits, random | 15.349 | 15.099 | 1.017x |
| blocks, 8 to 16 bits, 128-cell runs | 10.837 | 10.279 | 1.054x |
| biomes, 1 to 2 bits, up to three live ids | 0.307 | 0.249 | 1.230x |
| biomes, 2 to 3 bits, up to three live ids | 0.379 | 0.316 | 1.198x |
| biomes, 3 to 7 bits, up to three live ids | 0.280 | 0.312 | 0.898x |

Mixed block palettes benefit most: random 7-to-8-bit growth drops from
145.952 us to 17.601 us (8.292x). Long runs already benefit from the original
last-id cache, so gains there are much smaller. The global paths receive no
new cache and are not expected to improve; the biome global case above is
32 ns slower in this run. Results include gains and slowdowns, with no claim
that every input is faster. Repeat on target hardware before drawing
conclusions from sub-microsecond differences on a shared host.

These are palette-growth microbenchmarks, not world-generation timings or FPS.
There is no valid way to multiply the stage 1 and stage 2 ratios into a
whole-game speedup. Vanilla differential harnesses and an in-game benchmark
still require the player's prepared Minecraft files, unavailable here.

## Scope and next validation

This branch includes stage 1. Its PR targets `perf/bulk-palette-packing` so
reviewers see only stage 2; merge stage 1 before integrating this stage.
Next, validate real chunk edits and loading/saving with prepared game data,
then profile worker scheduling and render-frame spikes using `docs/benchmark.md`.
