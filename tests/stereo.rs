// Multi-view behavior: color masks, per-view depth clear, off-axis disparity.
mod common;
use common::*;
use scig::core::{
    pipeline::{
        DepthDesc,
        RenderPipelineDesc,
    },
    view::View,
};



const GB: wgpu::ColorWrites = wgpu::ColorWrites::GREEN.union(wgpu::ColorWrites::BLUE);
const R: wgpu::ColorWrites = wgpu::ColorWrites::RED;



#[tokio::test]
async fn single_view_writes_all_channels()
{
    let mut h = Harness::new().await;
    let mut tri = TrianglePass::new(h.solid_desc("fs", RGBA8));
    let out = h.render_rgba8((8, 8), &[cam_view()], &mut [&mut tri]);
    assert!(pixels(&out, 8).all(|(_, _, p)| p == WHITE));
}



#[tokio::test]
async fn two_masked_views_compose_to_white()
{
    // Failure signatures:
    //   [0,255,255,255]  second view cleared color.
    //   [255,0,0,255]    second view kept depth: equal z fails the Greater test.
    let mut h = Harness::new().await;
    let mut tri = TrianglePass::new(h.solid_desc("fs", RGBA8));
    let views = [cam_view().mask(R), cam_view().mask(GB)];
    let out = h.render_rgba8((8, 8), &views, &mut [&mut tri]);
    assert!(
        pixels(&out, 8).all(|(_, _, p)| p == WHITE),
        "{:?}",
        px(&out, 8, 0, 0)
    );
}



#[tokio::test]
async fn view_order_does_not_matter()
{
    let mut h = Harness::new().await;
    let mut tri = TrianglePass::new(h.solid_desc("fs", RGBA8));
    let views = [cam_view().mask(GB), cam_view().mask(R)];
    let out = h.render_rgba8((8, 8), &views, &mut [&mut tri]);
    assert!(pixels(&out, 8).all(|(_, _, p)| p == WHITE));
}



#[tokio::test]
async fn three_views_one_channel_each()
{
    let mut h = Harness::new().await;
    let mut tri = TrianglePass::new(h.solid_desc("fs", RGBA8));
    let views = [
        cam_view().mask(R),
        cam_view().mask(wgpu::ColorWrites::GREEN),
        cam_view().mask(wgpu::ColorWrites::BLUE),
    ];
    let out = h.render_rgba8((8, 8), &views, &mut [&mut tri]);
    assert!(pixels(&out, 8).all(|(_, _, p)| p == WHITE));
}



#[tokio::test]
async fn masked_view_leaves_other_channels_at_clear_value()
{
    let mut h = Harness::new().await;
    let mut tri = TrianglePass::new(h.solid_desc("fs", RGBA8));
    let out = h.render(
        (8, 8),
        RGBA8,
        wgpu::Color {
            r: 0.0,
            g: 0.0,
            b: 1.0,
            a: 1.0,
        },
        &[cam_view().mask(R)],
        &mut [&mut tri],
    );
    assert!(pixels(&out, 8).all(|(_, _, p)| p == [255, 0, 255, 255]));
}



#[tokio::test]
async fn views_with_different_matrices_produce_horizontal_disparity()
{
    // Eyes sit at x = -0.5 (red) and x = +0.5 (green+blue), parallel axes.
    // A point at x = 0 appears right of center for the left eye and left of
    // center for the right eye, so the red-only crescent is on the right.
    let mut h = Harness::new().await;
    let shader = h
        .shaders
        .load_shader_source(&h.context, include_str!("shaders/world_tri.wgsl"))
        .id();
    let desc = RenderPipelineDesc::new(shader, "vs", Some("fs"))
        .color_target(RGBA8, None)
        .depth(DepthDesc::reversed_z())
        .layout(h.view_buffer.layout().clone());
    let mut tri = TrianglePass::new(desc);
    tri.uses_view = true;

    let views = [
        View::new(eye_view(-0.5), proj90()).mask(R),
        View::new(eye_view(0.5), proj90()).mask(GB),
    ];
    let w = 48;
    let out = h.render_rgba8((w, w), &views, &mut [&mut tri]);

    let red_only = |p: [u8; 4]| p == [255, 0, 0, 255];
    let green_only = |p: [u8; 4]| p == [0, 255, 255, 255];
    let both = |p: [u8; 4]| p == WHITE;
    let count = |f: &dyn Fn([u8; 4]) -> bool| pixels(&out, w).filter(|(_, _, p)| f(*p)).count();

    assert!(count(&red_only) > 0, "no red-only pixels");
    assert!(count(&green_only) > 0, "no green-only pixels");
    assert!(count(&both) > 0, "views do not overlap");

    let cr = centroid_x(&out, w, red_only).unwrap();
    let cg = centroid_x(&out, w, green_only).unwrap();
    assert!(cr > cg, "red {cr} should be right of green {cg}");
    assert!(
        ((cr + cg) / 2.0 - 23.5).abs() < 1.0,
        "pair must be symmetric about the center"
    );
}



#[tokio::test]
async fn identical_views_have_zero_disparity()
{
    let mut h = Harness::new().await;
    let shader = h
        .shaders
        .load_shader_source(&h.context, include_str!("shaders/world_tri.wgsl"))
        .id();
    let desc = RenderPipelineDesc::new(shader, "vs", Some("fs"))
        .color_target(RGBA8, None)
        .depth(DepthDesc::reversed_z())
        .layout(h.view_buffer.layout().clone());
    let mut tri = TrianglePass::new(desc);
    tri.uses_view = true;
    let views = [
        View::new(eye_view(0.0), proj90()).mask(R),
        View::new(eye_view(0.0), proj90()).mask(GB),
    ];
    let w = 48;
    let out = h.render_rgba8((w, w), &views, &mut [&mut tri]);

    assert!(pixels(&out, w).all(|(_, _, p)| p == WHITE || p == BLACK));
    assert!(lit_count(&out, w) > 100);
}
