use horizon_shader_api::{Pack, StageKind, check};
use std::path::Path;

#[test]
fn optimized_presets_validate_and_keep_their_pass_budgets() {
    let packs = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/packs"));
    for (name, post, material) in [
        ("optimized_light", 0, 1),
        ("optimized_glow", 1, 0),
        ("golden_hour", 2, 0),
    ] {
        let report = check(&Pack::read(&packs.join(name)).unwrap()).unwrap();
        assert_eq!(
            report
                .stages
                .iter()
                .filter(|s| s.kind == StageKind::Post)
                .count(),
            post,
            "{name}"
        );
        assert_eq!(
            report
                .stages
                .iter()
                .filter(|s| s.kind == StageKind::Material)
                .count(),
            material,
            "{name}"
        );
        assert_eq!(
            report
                .stages
                .iter()
                .filter(|s| s.kind == StageKind::Sky)
                .count(),
            1,
            "{name}"
        );
    }
}
