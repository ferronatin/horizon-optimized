# OpenGL depth compatibility and efficient shader presets

The Intel HD 2500 report reaches world-ready with TAA disabled after GLSL rejects
WGSL depth textureLoad. Raw depth reads now sample the exact clamped texel centre
with a nearest, non-filtering sampler and explicit integer LOD 0. TAA and native
pack helpers share this approach. This does not change simulation calculations.

Naga 30 emits sampler2DShadow for these non-comparison depth reads. The vendored
GLES bridge corrects only reflected non-comparison, non-arrayed, single-sample
2D depth textureLod bindings to sampler2D and selects .r. Comparison sampling and
other texture types remain unchanged. Internal shader backend errors are caught
alongside validation errors so unsupported packs can fail gracefully.

| Included preset | Post passes | Bloom taps | Intermediate post targets |
| --- | ---: | ---: | ---: |
| Horizon Luz Rápida (profile default) | 0 | 0 | 0 |
| Horizon Leve Bonito | 1 | 4 | 0 |
| Golden Hour (original) | 2 | 12 | 1 |

Luz Rápida grades terrain colour in its existing material pass; both new presets
use a warm sky with sixth powers expanded into multiplications. Leve Bonito fuses
grade, glow and vignette and uses a simpler glow kernel. This trades effect detail
for fewer texture reads/passes, not pixel-identical output. Extra arithmetic still
has GPU cost. The presets add no dynamic shadows, reflections, SSAO or volumetrics.
The reversible profile selects Luz Rápida, fast clouds and mipmaps level 2. Original
Golden Hour is bundled for a fuller effect; options and pack selection remain exposed.
The native launcher UI is unchanged.

## Validation and limits

- All three packs pass the shader SDK validator/translators.
- 114 renderer graphics tests pass (7 ignored), including real available OpenGL
  pipeline compilation for all three packs, TAA pipeline/resource preparation,
  and exact nearest-depth gradient/edge readback across 16 pixels.
- Two GLES bridge tests pass via a rustc harness including the actual module and
  compiled Naga dependency; comparison samplers and similarly named user functions
  are preserved. The vendored HAL is not a workspace test package.
- Two reversible profile helper tests pass.

These are synthetic rendering fixtures, not a Minecraft session or an Intel HD
2500 measurement. There is no measured FPS improvement or native Windows visual
verification here. Test the same world, camera, resolution and pack for comparable
frame times on the affected PC. The earlier 16 MiB main-thread stack reserve and
world mesher changes remain included. No Mojang assets enter this package.
