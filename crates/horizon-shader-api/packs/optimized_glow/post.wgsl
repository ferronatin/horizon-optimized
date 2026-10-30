// Original Horizon colour grade plus a four-tap bloom, fused into one pass.
// No intermediate half-float render target, loop trigonometry or per-tap depth read.
override glow: f32 = 0.3;
override warmth: f32 = 0.4;
override saturation: f32 = 1.1;
override vignette: bool = true;

fn luminance(c: vec3<f32>) -> f32 {
    return dot(c, vec3<f32>(0.2126, 0.7152, 0.0722));
}
fn bright(uv: vec2<f32>) -> vec3<f32> {
    let c = textureSampleLevel(previous, linear_sampler, uv, 0.0).rgb;
    return c * smoothstep(0.8, 1.0, luminance(c));
}
fn post(input: PostInput) -> vec4<f32> {
    var c = input.color.rgb;
    if glow > 0.0 {
        let step = (frame.resolution.y / 720.0) / max(frame.resolution, vec2<f32>(1.0)) * 3.0;
        let sum = bright(input.uv + step) + bright(input.uv - step)
            + bright(input.uv + step * vec2<f32>(1.0, -1.0))
            + bright(input.uv + step * vec2<f32>(-1.0, 1.0));
        let sky = select(1.0, 0.25, input.depth == 0.0);
        c += sum * (glow * 0.375 * sky);
    }
    let day = select(0.0, smoothstep(-0.1, 0.1, frame.sun_direction.y), frame.dimension == 0);
    let low = 1.0 - smoothstep(0.0, 0.35, abs(frame.sun_direction.y));
    let golden = warmth * mix(0.3, 1.0, low) * day * (1.0 - frame.rain * 0.7);
    c *= mix(vec3<f32>(1.0), vec3<f32>(1.08, 1.0, 0.86), golden);
    c = c / (1.0 + 0.25 * c) * 1.25;
    c = mix(vec3<f32>(luminance(c)), c, saturation);
    if vignette {
        let aspect = frame.resolution.x / max(frame.resolution.y, 1.0);
        let p = (input.uv - 0.5) * vec2<f32>(aspect, 1.0);
        let radius_squared = dot(p, p) / (0.25 * (aspect * aspect + 1.0));
        c *= 1.0 - smoothstep(0.3025, 1.21, radius_squared) * 0.3;
    }
    return vec4<f32>(clamp(c, vec3<f32>(0.0), vec3<f32>(1.0)), input.color.a);
}
