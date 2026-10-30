// Shade in the existing terrain pass: no extra full-screen reads or targets.
// Keep vanilla alpha, tint and lightmap; use cheap colour arithmetic after them.
override warmth: f32 = 0.4;
override saturation: f32 = 1.08;
fn material(input: MaterialInput) -> Surface {
    let base = input.albedo * input.tint * vec4<f32>(input.lightmap, 1.0);
    let day = select(0.0, smoothstep(-0.1, 0.1, frame.sun_direction.y), frame.dimension == 0);
    let low = 1.0 - smoothstep(0.0, 0.35, abs(frame.sun_direction.y));
    let golden = warmth * mix(0.3, 1.0, low) * day * (1.0 - frame.rain * 0.7);
    var c = base.rgb * mix(vec3<f32>(1.0), vec3<f32>(1.08, 1.0, 0.86), golden);
    c = c / (1.0 + 0.25 * c) * 1.25;
    c = mix(vec3<f32>(dot(c, vec3<f32>(0.2126, 0.7152, 0.0722))), c, saturation);
    return Surface(vec4<f32>(clamp(c, vec3<f32>(0.0), vec3<f32>(1.0)), base.a));
}
