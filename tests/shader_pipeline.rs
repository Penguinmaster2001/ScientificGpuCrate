mod common;
use common::*;
use scig::core::{
    pipeline::{
        ComputePipelineDesc,
        DepthDesc,
        PipelineCache,
        RenderPipelineDesc,
    },
    shader::ShaderLib,
};
use std::sync::Arc;



const GOOD: &str = "@compute @workgroup_size(1) fn main() {}";
const OTHER: &str = "@compute @workgroup_size(2) fn main() {}";
const BAD: &str = "fn main( { this is not wgsl";



// ---------- ShaderLib ----------

#[tokio::test]
async fn shader_ids_are_content_addressed()
{
    let context = context().await;
    let mut lib = ShaderLib::new();
    let a = lib.load_shader_source(&context, GOOD).id();
    let b = lib.load_shader_source(&context, GOOD).id();
    let c = lib.load_shader_source(&context, OTHER).id();
    assert!(a == b, "same source must give the same id");
    assert!(a != c, "different source must give a different id");
    assert_eq!(lib.len(), 2, "duplicate source must not add an entry");
}



#[tokio::test]
async fn lookup_by_id()
{
    let context = context().await;
    let mut lib = ShaderLib::new();
    let id = lib.load_shader_source(&context, GOOD).id();
    assert!(lib.get_shader(id).is_some());
    let other = ShaderLib::new();
    assert!(other.get_shader(id).is_none());
}



#[tokio::test]
async fn file_and_source_with_same_text_share_an_id()
{
    let context = context().await;
    let path = std::env::temp_dir().join(format!("scig_{}.wgsl", std::process::id()));
    std::fs::write(&path, GOOD).unwrap();
    let mut lib = ShaderLib::new();
    let from_file = lib.load_shader_file(&context, &path).unwrap().id();
    let from_src = lib.load_shader_source(&context, GOOD).id();
    std::fs::remove_file(&path).ok();
    assert!(from_file == from_src);
}



#[tokio::test]
async fn missing_file_is_an_error_not_a_panic()
{
    let context = context().await;
    let mut lib = ShaderLib::new();
    assert!(
        lib.load_shader_file(&context, std::path::Path::new("/definitely/not/here.wgsl"))
            .is_err()
    );
}



#[tokio::test]
async fn invalid_wgsl_is_an_error_and_leaves_the_library_intact()
{
    let context = context().await;
    let mut lib = ShaderLib::new();
    let good = lib.load_shader_source(&context, GOOD).id();
    let before = lib.len();
    let err = lib.try_load_shader_source(&context, BAD).await;
    assert!(err.is_err());
    assert!(!err.err().unwrap().to_string().is_empty());
    assert_eq!(lib.len(), before);
    assert!(lib.get_shader(good).is_some());
    assert!(
        lib.try_load_shader_source(&context, OTHER).await.is_ok(),
        "library must stay usable"
    );
}



// ---------- PipelineCache ----------

fn base(h: &mut Harness) -> RenderPipelineDesc
{
    h.solid_desc("fs", RGBA8)
}



#[tokio::test]
async fn identical_descriptors_share_one_pipeline()
{
    let mut h = Harness::new().await;
    let desc = base(&mut h);
    let mut cache = PipelineCache::new();
    let a = cache.render(&h.context, &h.shaders, &desc).unwrap();
    let b = cache.render(&h.context, &h.shaders, &desc.clone()).unwrap();
    assert!(Arc::ptr_eq(&a, &b));
    assert_eq!(cache.len(), 1);
}



#[tokio::test]
async fn every_pipeline_state_is_part_of_the_key()
{
    let mut h = Harness::new().await;
    let base = base(&mut h);
    let other_shader = h
        .shaders
        .load_shader_source(&h.context, &format!("{SOLID}\n// x"))
        .id();

    let mut no_cull = base.clone();
    no_cull.cull = None;
    let mut lines = base.clone();
    lines.topology = wgpu::PrimitiveTopology::LineList;
    let mut other = base.clone();
    other.shader = other_shader;

    let variants: Vec<(&str, RenderPipelineDesc)> = vec![
        ("front_face", base.clone().front_face(wgpu::FrontFace::Cw)),
        ("cull", no_cull),
        ("topology", lines),
        (
            "fragment entry",
            RenderPipelineDesc::new(base.shader, "vs", Some("fs_half"))
                .color_target(RGBA8, None)
                .depth(DepthDesc::reversed_z()),
        ),
        (
            "format",
            RenderPipelineDesc::new(base.shader, "vs", Some("fs"))
                .color_target(wgpu::TextureFormat::Bgra8Unorm, None)
                .depth(DepthDesc::reversed_z()),
        ),
        (
            "blend",
            RenderPipelineDesc::new(base.shader, "vs", Some("fs"))
                .color_target(RGBA8, Some(wgpu::BlendState::ALPHA_BLENDING))
                .depth(DepthDesc::reversed_z()),
        ),
        (
            "color mask",
            base.clone().color_mask(wgpu::ColorWrites::RED),
        ),
        (
            "depth",
            RenderPipelineDesc::new(base.shader, "vs", Some("fs"))
                .color_target(RGBA8, None)
                .depth(DepthDesc::always()),
        ),
        ("shader", other),
        (
            "layout",
            base.clone().layout(h.view_buffer.layout().clone()),
        ),
    ];

    let mut cache = PipelineCache::new();
    let reference = cache.render(&h.context, &h.shaders, &base).unwrap();
    for (name, desc) in &variants
    {
        let p = cache.render(&h.context, &h.shaders, desc).unwrap();
        assert!(
            !Arc::ptr_eq(&reference, &p),
            "{name} must change the cache key"
        );
    }
    assert_eq!(cache.len(), 1 + variants.len());
}



#[tokio::test]
async fn multiple_color_targets()
{
    let mut h = Harness::new().await;
    let desc = RenderPipelineDesc::new(base(&mut h).shader, "vs", Some("fs_mrt"))
        .color_target(RGBA8, None)
        .color_target(RGBA8, None)
        .depth(DepthDesc::reversed_z());
    let mut cache = PipelineCache::new();
    cache
        .render(&h.context, &h.shaders, &desc)
        .expect("two targets");
}



#[tokio::test]
async fn depth_only_pipeline_has_no_fragment_stage()
{
    let mut h = Harness::new().await;
    let desc =
        RenderPipelineDesc::new(base(&mut h).shader, "vs", None).depth(DepthDesc::reversed_z());
    let mut cache = PipelineCache::new();
    cache
        .render(&h.context, &h.shaders, &desc)
        .expect("depth only");
}



#[tokio::test]
async fn missing_entry_point_is_an_error()
{
    let mut h = Harness::new().await;
    let shader = base(&mut h).shader;
    let desc = RenderPipelineDesc::new(shader, "nope", Some("fs")).color_target(RGBA8, None);
    let mut cache = PipelineCache::new();
    assert!(cache.render(&h.context, &h.shaders, &desc).is_err());
    assert_eq!(cache.len(), 0, "failed creation must not be cached");
}



#[tokio::test]
async fn compute_pipeline_from_desc()
{
    let mut h = Harness::new().await;
    let id = h.shaders.load_shader_source(&h.context, GOOD).id();
    ComputePipelineDesc::new(id)
        .create_pipeline(&h.context, &h.shaders)
        .expect("compute pipeline");
}



// ---------- state that must reach the rasterizer ----------

#[tokio::test]
async fn color_mask_limits_written_channels()
{
    let mut h = Harness::new().await;
    let desc = h.solid_desc("fs", RGBA8);
    let mut tri = TrianglePass::new(desc);
    let out = h.render_rgba8(
        (4, 4),
        &[cam_view().mask(wgpu::ColorWrites::RED)],
        &mut [&mut tri],
    );
    assert!(pixels(&out, 4).all(|(_, _, p)| p == RED));
}



#[tokio::test]
async fn cull_none_draws_back_faces()
{
    let mut h = Harness::new().await;
    let mut desc = h.solid_desc("fs", RGBA8).front_face(wgpu::FrontFace::Cw);
    let mut culled = TrianglePass::new(desc.clone());
    let out = h.render_rgba8((4, 4), &[cam_view()], &mut [&mut culled]);
    assert!(
        pixels(&out, 4).all(|(_, _, p)| p == BLACK),
        "Cw + Back must cull"
    );

    desc.cull = None;
    let mut visible = TrianglePass::new(desc);
    let out = h.render_rgba8((4, 4), &[cam_view()], &mut [&mut visible]);
    assert!(pixels(&out, 4).all(|(_, _, p)| p == WHITE));
}



fn additive() -> wgpu::BlendState
{
    let c = wgpu::BlendComponent {
        src_factor: wgpu::BlendFactor::One,
        dst_factor: wgpu::BlendFactor::One,
        operation: wgpu::BlendOperation::Add,
    };
    wgpu::BlendState { color: c, alpha: c }
}



#[tokio::test]
async fn additive_blend_accumulates_and_replace_does_not()
{
    let mut h = Harness::new().await;
    let shader = h.solid_desc("fs", RGBA8).shader;
    let mk = |blend| {
        RenderPipelineDesc::new(shader, "vs", Some("fs_half"))
            .color_target(RGBA8, blend)
            .depth(DepthDesc::always())
    };

    let mut add = TrianglePass::new(mk(Some(additive())));
    add.instances = 2;
    let out = h.render_rgba8((4, 4), &[cam_view()], &mut [&mut add]);
    assert!(
        px(&out, 4, 1, 1)[0] >= 254,
        "0.5 + 0.5 should saturate: {:?}",
        px(&out, 4, 1, 1)
    );

    let mut replace = TrianglePass::new(mk(None));
    replace.instances = 2;
    let out = h.render_rgba8((4, 4), &[cam_view()], &mut [&mut replace]);
    assert!(px(&out, 4, 1, 1)[0].abs_diff(128) <= 1);
}



#[tokio::test]
async fn depth_greater_with_equal_depth_rejects_second_draw()
{
    // Two instances at identical z: the second fails Greater. With Always it passes.
    let mut h = Harness::new().await;
    let shader = h.solid_desc("fs", RGBA8).shader;
    let mk = |depth, blend| {
        RenderPipelineDesc::new(shader, "vs", Some("fs_half"))
            .color_target(RGBA8, blend)
            .depth(depth)
    };
    let mut strict = TrianglePass::new(mk(DepthDesc::reversed_z(), Some(additive())));
    strict.instances = 2;
    let out = h.render_rgba8((4, 4), &[cam_view()], &mut [&mut strict]);
    assert!(
        px(&out, 4, 1, 1)[0].abs_diff(128) <= 1,
        "second draw must be rejected"
    );
}
