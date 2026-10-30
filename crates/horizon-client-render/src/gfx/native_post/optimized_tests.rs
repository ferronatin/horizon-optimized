//! Native GL checks for the shipped quality/performance presets.
use crate::gfx::{native_material::TerrainShape, native_pack::NativePack};
use horizon_shader_api::{Pack, check};
use std::path::Path;

fn gl_device() -> Option<(wgpu::Device, wgpu::Queue)> {
    let mut desc = wgpu::InstanceDescriptor::new_without_display_handle();
    desc.backends = wgpu::Backends::GL;
    let instance = wgpu::Instance::new(desc);
    let adapter = pollster::block_on(instance.request_adapter(&Default::default())).ok()?;
    pollster::block_on(adapter.request_device(&Default::default())).ok()
}

fn globals_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    let mut entries = Vec::new();
    let mut entry = |binding, ty| {
        entries.push(wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
            ty,
            count: None,
        })
    };
    entry(
        0,
        wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
    );
    for binding in [2, 4] {
        entry(
            binding,
            wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
        );
    }
    entry(
        3,
        wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        },
    );
    for binding in [1, 5, 6, 7, 8, 9, 10, 11] {
        entry(
            binding,
            wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2Array,
                multisampled: false,
            },
        );
    }
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Preset terrain fixture"),
        entries: &entries,
    })
}

#[test]
fn presets_compile_on_gl_and_only_full_preset_allocates_intermediate_post_targets() {
    let Some((device, _)) = gl_device() else {
        return;
    };
    let layout = globals_layout(&device);
    let shape = TerrainShape {
        globals: &layout,
        flat: None,
        sprites: [1.0, 2048.0],
        format: wgpu::TextureFormat::Rgba8Unorm,
    };
    for name in ["optimized_light", "optimized_glow", "golden_hour"] {
        let path = Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../horizon-shader-api/packs"
        ))
        .join(name);
        let pack = Pack::read(&path).unwrap();
        let report = check(&pack).unwrap();
        let constants = pack
            .manifest
            .options
            .iter()
            .map(|o| (o.id.clone(), o.kind.default_constant()))
            .collect::<Vec<_>>();
        let (mut native, _) = NativePack::new(&device, &shape, report, &constants)
            .unwrap_or_else(|e| panic!("{name}: {e}"));
        assert!(native.sky.is_some());
        assert_eq!(native.material.is_some(), name == "optimized_light");
        assert_eq!(native.post.is_none(), name == "optimized_light");
        if let Some(post) = &mut native.post {
            let texture = |format| {
                device
                    .create_texture(&wgpu::TextureDescriptor {
                        label: None,
                        size: wgpu::Extent3d {
                            width: 4,
                            height: 4,
                            depth_or_array_layers: 1,
                        },
                        mip_level_count: 1,
                        sample_count: 1,
                        dimension: wgpu::TextureDimension::D2,
                        format,
                        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                            | wgpu::TextureUsages::TEXTURE_BINDING,
                        view_formats: &[],
                    })
                    .create_view(&Default::default())
            };
            let color = texture(shape.format);
            let depth = texture(wgpu::TextureFormat::Depth32Float);
            post.prepare(&device, (&color, &depth), [4, 4]);
            assert_eq!(post.targets.is_some(), name == "golden_hour");
        }
    }
}
