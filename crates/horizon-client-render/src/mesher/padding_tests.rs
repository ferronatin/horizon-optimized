//! Compare the row copy to the shipped per-cell padding builder.
use super::*;
use crate::chunk::{ColumnLayout, Light, Section};
use std::{hint::black_box, sync::Arc, time::Instant};

fn reference(region: &[[Option<&Chunk>; 3]; 3], sy: i32) -> Padded {
    let mut blocks = vec![0; P * P * P];
    let mut light = vec![0; P * P * P];
    let base_y = region[1][1].map_or(MIN_Y, Chunk::min_y) + sy * 16;
    let mut own = Box::new([0; 4096]);
    if let Some(section) = region[1][1].map(|c| &c.sections[sy as usize]) {
        section.unpack(&mut own);
    }
    for y in -B..16 + B {
        let wy = base_y + y;
        for z in -B..16 + B {
            for x in -B..16 + B {
                let i = pi(x, y, z);
                if let Some(c) = region[((z + 16) >> 4) as usize][((x + 16) >> 4) as usize] {
                    blocks[i] =
                        if (0..16).contains(&x) && (0..16).contains(&y) && (0..16).contains(&z) {
                            own[(y << 8 | z << 4 | x) as usize]
                        } else {
                            c.get(x & 15, wy, z & 15)
                        };
                    light[i] = c.light(x & 15, wy, z & 15);
                } else {
                    light[i] = 0xF0;
                }
            }
        }
    }
    Padded { blocks, light }
}

fn region(mode: usize, missing: u16) -> [[Option<Chunk>; 3]; 3] {
    std::array::from_fn(|z| {
        std::array::from_fn(|x| {
            let n = z * 3 + x;
            if missing & (1 << n) != 0 {
                return None;
            }
            let low = if mode == 4 {
                (n as i32 % 3 - 1) * 32
            } else {
                -64
            };
            let layout = ColumnLayout::new(low, if mode == 4 { 48 } else { 64 }).unwrap();
            let mut chunk = Chunk::with_layout(x as i32 - 1, z as i32 - 1, layout);
            for (si, section) in chunk.sections.iter_mut().enumerate() {
                let values = std::array::from_fn(|i| match mode {
                    0 => 1,
                    1 => 0,
                    3 => (i * 17 + si * 13 + n * 31) as u32 % 600,
                    _ => ((i * 17 + si * 13 + n * 31) % 7) as u32,
                });
                *section = Arc::new(Section {
                    blocks: Section::from_blocks(&values),
                    light: if mode <= 1 {
                        Light::Uniform((si * 37 + n) as u8)
                    } else {
                        Light::Full(Box::new(std::array::from_fn(|i| {
                            (i * 19 + si + n * 11) as u8
                        })))
                    },
                    ..Section::default()
                });
            }
            Some(chunk)
        })
    })
}

#[test]
fn rows_match_every_block_and_light_including_borders_and_build_limits() {
    for mode in 0..=4 {
        // All nine neighbours present, missing sides/corners, and no centre.
        for missing in [0, 1, 0b101_000_101, 0b010_101_010, 0b111_111_111] {
            let chunks = region(mode, missing);
            let refs = std::array::from_fn(|z| std::array::from_fn(|x| chunks[z][x].as_ref()));
            for sy in 0..chunks[1][1].as_ref().map_or(4, |c| c.sections.len()) {
                let old = reference(&refs, sy as i32);
                let new = Padded::new(&refs, sy as i32);
                assert_eq!(
                    new.blocks, old.blocks,
                    "blocks: mode={mode}, missing={missing}, sy={sy}"
                );
                assert_eq!(
                    new.light, old.light,
                    "light: mode={mode}, missing={missing}, sy={sy}"
                );
            }
        }
    }
}

#[test]
#[ignore = "opt-in release microbenchmark, not a game FPS measurement"]
fn benchmark_padding_rows() {
    for (name, mode) in [("uniform", 0), ("mixed_local", 2), ("mixed_global", 3)] {
        let chunks = region(mode, 0);
        let refs = std::array::from_fn(|z| std::array::from_fn(|x| chunks[z][x].as_ref()));
        let run = |f: fn(&[[Option<&Chunk>; 3]; 3], i32) -> Padded| {
            let start = Instant::now();
            for _ in 0..2000 {
                black_box(f(black_box(&refs), black_box(1)));
            }
            start.elapsed().as_nanos() as f64 / 2000.0
        };
        let mut old = Vec::new();
        let mut new = Vec::new();
        for i in 0..9 {
            if i % 2 == 0 {
                old.push(run(reference));
                new.push(run(Padded::new));
            } else {
                new.push(run(Padded::new));
                old.push(run(reference));
            }
        }
        old.sort_by(f64::total_cmp);
        new.sort_by(f64::total_cmp);
        eprintln!(
            "padding {name}: old={:.0} ns new={:.0} ns speedup={:.2}x",
            old[4],
            new[4],
            old[4] / new[4]
        );
    }
}
