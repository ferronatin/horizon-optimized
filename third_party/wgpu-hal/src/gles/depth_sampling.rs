//! Horizon's Naga 30 compatibility bridge for non-comparison depth LOD reads.
//! Naga declares every depth image as a shadow sampler, even with a regular
//! sampler. GLSL shadow samplers require a comparison reference and cannot
//! implement WGSL's scalar raw-depth read. Keep reflection/bindings unchanged;
//! regular 2D depth LOD reads use sampler2D's red component instead.

use alloc::{format, string::String};

pub(super) fn fix_raw_depth_lod(
    source: &mut String,
    module: &naga::Module,
    reflection: &naga::back::glsl::ReflectionInfo,
) {
    let mut definitions = String::new();
    let mut id = 0;
    for (name, mapping) in &reflection.texture_mapping {
        let image = &module.global_variables[mapping.texture];
        let Some(sampler) = mapping.sampler else {
            continue;
        };
        if !matches!(
            module.types[image.ty].inner,
            naga::TypeInner::Image {
                dim: naga::ImageDimension::D2,
                arrayed: false,
                class: naga::ImageClass::Depth { multi: false }
            }
        ) || !matches!(
            module.types[module.global_variables[sampler].ty].inner,
            naga::TypeInner::Sampler { comparison: false }
        ) {
            continue;
        }
        let declaration = format!("sampler2DShadow {name};");
        let call = format!("textureLod({name},");
        if !source.contains(&declaration) || !source.contains(&call) {
            continue;
        }
        let helper = loop {
            let candidate = format!("horizonRawDepthLod{id}");
            id += 1;
            if !source.contains(&candidate) {
                break candidate;
            }
        };
        let rewritten = replace_call(source, &call, &format!("{helper}("));
        if rewritten == *source {
            continue;
        }
        *source = rewritten.replace(&declaration, &format!("sampler2D {name};"));
        definitions.push_str(&format!(
            "#define {helper}(uv, level) (textureLod({name}, (uv), float(level)).r)\n"
        ));
    }
    if !definitions.is_empty() {
        // #version must stay first. A macro adds no run-time shader work and
        // can precede extension/precision declarations without needing types.
        if let Some(line) = source.find('\n') {
            source.insert_str(line + 1, &definitions);
        }
    }
}

// Match a built-in token, not the suffix of a user function such as _textureLod.
fn replace_call(source: &str, call: &str, replacement: &str) -> String {
    let mut result = String::with_capacity(source.len());
    let mut copied = 0;
    for (at, _) in source.match_indices(call) {
        if at > 0
            && (source.as_bytes()[at - 1].is_ascii_alphanumeric()
                || source.as_bytes()[at - 1] == b'_')
        {
            continue;
        }
        result.push_str(&source[copied..at]);
        result.push_str(replacement);
        copied = at + call.len();
    }
    result.push_str(&source[copied..]);
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec::Vec;
    fn fixture(comparison: bool) -> (naga::Module, naga::back::glsl::ReflectionInfo) {
        let mut m = naga::Module::default();
        let image = m.types.insert(
            naga::Type {
                name: None,
                inner: naga::TypeInner::Image {
                    dim: naga::ImageDimension::D2,
                    arrayed: false,
                    class: naga::ImageClass::Depth { multi: false },
                },
            },
            naga::Span::UNDEFINED,
        );
        let sampler = m.types.insert(
            naga::Type {
                name: None,
                inner: naga::TypeInner::Sampler { comparison },
            },
            naga::Span::UNDEFINED,
        );
        let mut global = |ty| {
            m.global_variables.append(
                naga::GlobalVariable {
                    name: None,
                    space: naga::AddressSpace::Handle,
                    binding: None,
                    ty,
                    init: None,
                    memory_decorations: Default::default(),
                },
                naga::Span::UNDEFINED,
            )
        };
        let (texture, sampler) = (global(image), global(sampler));
        let mut r = naga::back::glsl::ReflectionInfo {
            texture_mapping: Default::default(),
            uniforms: Default::default(),
            varying: Default::default(),
            immediates_items: Vec::new(),
            clip_distance_count: 0,
        };
        r.texture_mapping.insert(
            "depthTex".into(),
            naga::back::glsl::TextureMapping {
                texture,
                sampler: Some(sampler),
            },
        );
        (m, r)
    }
    #[test]
    fn comparison_sampling_stays_unchanged() {
        let (m, r) = fixture(true);
        let mut source = String::from("#version 330 core\nuniform sampler2DShadow depthTex;\nfloat v = textureLod(depthTex, vec3(0.5), 0.0);\n");
        let original = source.clone();
        fix_raw_depth_lod(&mut source, &m, &r);
        assert_eq!(source, original);
    }
    #[test]
    fn raw_depth_changes_type_and_extracts_red_without_touching_user_functions() {
        let (m, r) = fixture(false);
        let mut source = String::from("#version 330 core\nuniform sampler2DShadow depthTex;\nfloat v = textureLod(depthTex, uv, 0);\n_textureLod(depthTex, uv, 0);\n");
        fix_raw_depth_lod(&mut source, &m, &r);
        assert!(source.starts_with("#version 330 core\n#define "));
        assert!(source.contains("uniform sampler2D depthTex;"));
        assert!(source.contains("float(level)).r"));
        assert!(source.contains("_textureLod(depthTex, uv, 0)"));
        assert!(!source.contains("float v = textureLod"));
    }
}
