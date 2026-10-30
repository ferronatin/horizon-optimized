# Windows installer with a disk selector

The Windows MSVC client now reserves 16 MiB for its main-thread stack, retaining
the 4 KiB initial commit. The previous executable used the linker's 1 MiB
default; a Windows 10 / Intel HD 2500 report hit `0xc00000fd` while recreating
the loading renderer. The packager rejects clients with the old small reserve.
This changes no launcher UI or game settings. Native validation of that report
still requires the affected machine; the optional OpenGL TAA warning has its
own fallback and is not a fatal error in the supplied log.

The legacy edition now opens native NSIS controls, not GPUI. Sign-in and play
invoke the unmodified official launcher's CLI in a visible console. The full
launcher remains available through a button and `Horizon-Completo.exe`.
The Rust profile helper merges only performance keys, backs up originals once,
and restores exact file bytes/missing files. Its unit tests also verify worlds
are untouched. Build it separately with its Cargo manifest under `profile-tool/`
for Windows x64 and place `horizon-light-profile.exe` beside the engine/CLI.

Default legacy settings are explicit/reversible: OpenGL, 960x540, 6 render chunks,
5 simulation chunks, 2 workers, no blur/HDR, fast clouds, mipmaps level 2,
Horizon Luz Rápida shader, static panorama and 60 FPS
in play. The loading/title loop now yields using event-loop deadlines rather
than continuously polling, caps foreground menus at 60 by default, and accepts
`HORIZON_MENU_FPS=30` from the light front door. Offscreen scripted rendering
keeps the old unrestricted path. Other keyboard, networking and game rules stay.
The old game engine source commit mentioned below identifies the original two
palette optimizations; current build provenance is recorded in BUILD-INFO.json.

`Horizon-Optimized-Setup.exe` is a per-user NSIS installer with a Portuguese
directory page. It bundles the optimized x64 client/CLI from this fork and the
unmodified, Microsoft-configured official Horizon launcher. It never copies
another application's OAuth client id into a new launcher build.

The installed `Horizon-Optimized.exe` front door sets the two Horizon homes,
temporary directories and native-mod toolchain caches under the chosen folder.
Only the two Horizon-specific variables are persisted to HKCU for instance
shortcuts; previous values are saved and restored on uninstall if still ours.
TEMP, TMP, CARGO_HOME and RUSTUP_HOME are never changed system-wide. The wizard
explains that the chosen folder applies to other Horizon shortcuts for this user.
Existing worlds are not moved/deleted, settings survive reinstalls, and the data
folder survives uninstall. Ordinary OS credentials remain in the Windows vault.

Automatic game installs are off in the initial settings, and the front door
turns off launcher self-replacement so upstream updates do not silently replace
this edition. Manual upstream versions remain available in the official launcher.

The installer bundles three original Horizon-format shader presets. Luz Rápida
grades terrain in its existing material pass with a warm sky and no post pass;
Leve Bonito adds a four-tap glow and grade in one post pass without intermediate
post targets; Golden Hour retains the original two-pass, twelve-tap effect.
The profile selects Luz Rápida; players can select or disable packs in game.
These presets do not add dynamic shadows, reflections, SSAO or volumetrics.
See [the validation report](../../docs/performance/gl-shaders-lite.md).

## Rebuild

Build `horizon-client` and `horizon-ctl` for Windows x64 with the static C runtime
(`RUSTFLAGS=-C target-feature=+crt-static`). Both optimizations are in source
commit `cfcd3a02867b170b196d1572c13f3f8f0306eda9`.

Install NSIS 3.09+ and Python's optional `cryptography` package. Download the
Windows game archive, SHA256SUMS and signature from official
`nightly-2026-10-30`, and the launcher archive/checksums/signature from
`launcher@0.2.28`. The packager refuses inputs with invalid signatures or hashes.

```sh
python tools/package_optimized_windows.py \
  --game game.zip --game-sums game-SHA256SUMS --game-signature game-SHA256SUMS.sig \
  --launcher launcher.zip --launcher-sums launcher-SHA256SUMS \
  --launcher-signature launcher-SHA256SUMS.sig \
  --engine-dir target/x86_64-pc-windows-msvc/release \
  --makensis /path/to/makensis --out .local/optimized-windows \
  --source-commit cfcd3a02867b170b196d1572c13f3f8f0306eda9
```

Only original Horizon exporters/support programs are reused; no Minecraft jars,
prepared files, worlds or account data enter the installer. `BUILD-INFO.json`
records input versions, archive hashes, engine source and payload hashes.

Validation performed: Windows x64 release build with both optimized binaries,
PE import checks (system DLLs only), official Ed25519/SHA-256 verification,
NSIS compilation and Python packager/isolation tests. The full Python suite
retains the pre-existing RSS failure documented in the first two PRs. This
Linux environment did not execute the installer UI or run a Minecraft session
on Windows; native runtime testing remains necessary.
