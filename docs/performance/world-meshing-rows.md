# Terrain padding and legacy upload scheduling

The section mesher prepares a 20x20x20 neighbourhood of block IDs and packed
light before computing visibility, faces and shading. Previously every cell
resolved a chunk and called `Chunk::light`, repeating the section lookup and
light representation branch. The new builder resolves one contiguous row span
at a time. Uniform light is filled, full light is copied, and single-value
border palettes are filled directly. The centre retains its existing bulk
palette unpack. All geometry and lighting algorithms after this step are intact.

The differential test compares every block and light byte of 96 section buffers
against the shipped builder: uniform air/stone, local/global palettes, full and
uniform light, missing sides/corners/centre, differing column layouts, and
bottom/top build limits. It needs no Minecraft assets or prepared world data.

`FaceVisibility` workers now share immutable state/shape tables through `Arc`.
Only the bounded, mutable pair cache remains private to each clone. Its lifecycle
test checks independent caches, identical visibility answers, and survival after
the source table is dropped (an in-flight worker during resource reload).

The reversible Windows light profile now enables existing upload budgets:
`chunkUpdates:4` and `uploadBudgetKiB:512`. Previously both were unlimited.
This distributes terrain uploads across frames on older OpenGL drivers. A single
oversized mesh is still allowed through to prevent starvation. Under heavy
exploration, terrain may appear more gradually. Simulation distance, ticks,
generation, saving and gameplay rules are unchanged by these scheduling keys.
The installer/launcher interface is unchanged; the existing backup can restore
original settings.

## Measurement

Linux x86-64 AMD EPYC 9V74 VM, Rust 1.99.1, optimized `fast` profile (no LTO).
Nine alternating before/after samples, 2,000 constructions each; median below.
Other build jobs ran concurrently. Raw results and limitations:
`benchmarks/world-meshing-rows-2026-10-30.json`.

| Padding fixture | Before | After | Speedup |
| --- | ---: | ---: | ---: |
| Uniform blocks/light | 39.006 us | 15.830 us | 2.46x |
| Mixed local palette/full light | 55.331 us | 35.835 us | 1.54x |
| Mixed global palette/full light | 54.187 us | 35.300 us | 1.54x |

These are padding-stage microbenchmarks, not full chunk generation/meshing or
in-game FPS results. Native Intel HD 2500 / i5 validation remains necessary.
No game systems, visual features or objects are removed by the row-copy change.

```sh
cargo test --locked --profile fast -p horizon-client-render --lib \
  mesher::padding_tests::benchmark_padding_rows -- --ignored --nocapture
```
