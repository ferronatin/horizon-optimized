// Horizon's original Golden Hour look, with the sixth power expanded.
fn sky(input: SkyInput) -> vec4<f32> {
    let d = input.direction;
    let toward = max(dot(d, frame.sun_direction), 0.0);
    let squared = toward * toward;
    let horizon = 1.0 - smoothstep(0.0, 0.4, abs(d.y));
    let day = select(0.0, smoothstep(-0.1, 0.1, frame.sun_direction.y), frame.dimension == 0)
        * (1.0 - frame.rain);
    let low = 1.0 - smoothstep(0.0, 0.35, abs(frame.sun_direction.y));
    let halo = squared * squared * squared * 0.6 + horizon * squared * 0.4;
    let warm = vec3<f32>(1.0, 0.55, 0.25) * halo * low * day * 0.6;
    let zenith = smoothstep(0.1, 1.0, d.y) * day * 0.12;
    return vec4<f32>(input.color.rgb * (1.0 - zenith * vec3<f32>(0.5, 0.25, 0.0)) + warm, input.color.a);
}
