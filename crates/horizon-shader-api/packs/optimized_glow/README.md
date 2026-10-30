# Horizon Leve Bonito

Original Horizon-format preset based on Horizon's MIT-licensed Golden Hour.
One fused post pass applies a four-tap glow, colour grade and squared-radius
vignette. The sky expands the sixth power into multiplications. Compared with
Golden Hour's two post passes/twelve bloom taps, this variant uses fewer reads
and no intermediate post targets; its glow is simpler and its vignette differs.
It does not add dynamic shadow maps, reflections, SSAO or volumetric lighting.
`budget.gpu_ms` is a target for the overlay, not a measured performance claim.
Disable or adjust effects through the manifest options. Actual FPS depends on
the world, resolution and driver.
