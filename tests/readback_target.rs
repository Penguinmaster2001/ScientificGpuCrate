mod common;



use common::*;
use scig::core::{
    pass::{
        PassOps,
        begin_render_pass,
    },
    readback::{
        EncoderToken,
        TextureReadBack,
    },
    target::Target,
    texture::Texture,
};



fn clear_and_read(
    context: &scig::core::context::Context,
    size: (u32, u32),
    format: wgpu::TextureFormat,
    color: wgpu::Color,
) -> Vec<u8>
{
    let mut target = Target::offscreen(context, size, format);
    let frame = target.acquire(context).unwrap();
    let mut enc = context.device.create_command_encoder(&Default::default());
    drop(begin_render_pass(
        &mut enc,
        &frame,
        PassOps::clear(color, 0.0),
    ));
    let rb = TextureReadBack::encode_texture(context, &mut enc, target.color().unwrap());
    context.queue.submit([enc.finish()]);
    rb.map_texture(context, EncoderToken { id: 0 }).unwrap()
}



// ---------- TextureReadBack ----------

#[tokio::test]
async fn r8_rows_are_tight_for_unaligned_widths()
{
    let context = context().await;
    for w in [1u32, 3, 5, 255, 256, 257]
    {
        let c = wgpu::Color {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        };
        let out = clear_and_read(&context, (w, 3), wgpu::TextureFormat::R8Unorm, c);
        assert_eq!(out.len(), (w * 3) as usize, "width {w}");
        assert!(out.iter().all(|b| *b == 255), "width {w}");
    }
}



#[tokio::test]
async fn rgba16f_uses_eight_bytes_per_pixel()
{
    let context = context().await;
    let c = wgpu::Color {
        r: 0.5,
        g: 0.25,
        b: 0.0,
        a: 1.0,
    };
    for w in [1u32, 33]
    {
        let out = clear_and_read(&context, (w, 2), wgpu::TextureFormat::Rgba16Float, c);
        assert_eq!(out.len(), (w * 2 * 8) as usize);
        // f16 little endian: 0.5 = 0x3800, 0.25 = 0x3400, 0 = 0x0000, 1 = 0x3C00
        let px: [u8; 8] = [0x00, 0x38, 0x00, 0x34, 0x00, 0x00, 0x00, 0x3C];
        assert!(out.as_chunks::<8>().0.iter().all(|p| *p == px), "width {w}");
    }
}



#[tokio::test]
async fn every_pixel_lands_at_its_own_coordinates()
{
    // fs_coords writes [x, y, 0, 255]. A wrong row stride or a flipped y shows up here.
    let mut h = Harness::new().await;
    for (w, hgt) in [(5u32, 3u32), (33, 7), (64, 64), (255, 2)]
    {
        let mut tri = TrianglePass::new(h.solid_desc("fs_coords", RGBA8));
        let out = h.render_rgba8((w, hgt), &[cam_view()], &mut [&mut tri]);
        assert_eq!(out.len(), (w * hgt * 4) as usize);
        for (x, y, p) in pixels(&out, w)
        {
            assert!(
                near(p, [x as u8, y as u8, 0, 255], 1),
                "({x},{y}) = {p:?} in {w}x{hgt}"
            );
        }
    }
}



// ---------- Target ----------

#[tokio::test]
async fn offscreen_frame_describes_the_target()
{
    let context = context().await;
    let mut t = Target::offscreen(&context, (7, 5), RGBA8);
    let f = t.acquire(&context).unwrap();
    assert_eq!(f.size, [7, 5]);
    assert_eq!(f.format, RGBA8);
    assert_eq!(t.color().unwrap().size(), (7, 5));
}



#[tokio::test]
async fn resize_recreates_color_and_depth_together()
{
    let context = context().await;
    let mut t = Target::offscreen(&context, (8, 8), RGBA8);
    let g0 = t.generation();
    t.resize(&context, (5, 3));
    assert_eq!(t.generation(), g0 + 1);
    assert_eq!(t.color().unwrap().size(), (5, 3));
    assert_eq!(t.depth().unwrap().size(), (5, 3));
    let out = clear_and_read(&context, (5, 3), RGBA8, wgpu::Color::BLACK);
    assert_eq!(out.len(), 5 * 3 * 4);
}



#[tokio::test]
async fn resize_to_same_size_is_a_no_op()
{
    let context = context().await;
    let mut t = Target::offscreen(&context, (8, 8), RGBA8);
    let g0 = t.generation();
    t.resize(&context, (8, 8));
    assert_eq!(t.generation(), g0);
}



#[tokio::test]
async fn resize_to_zero_is_ignored()
{
    // Minimizing a window delivers 0x0.
    let context = context().await;
    let mut t = Target::offscreen(&context, (8, 8), RGBA8);
    let g0 = t.generation();
    t.resize(&context, (0, 0));
    t.resize(&context, (8, 0));
    t.resize(&context, (0, 8));
    assert_eq!(t.generation(), g0);
    assert_eq!(t.color().unwrap().size(), (8, 8));
}



#[tokio::test]
async fn present_on_offscreen_frame_is_a_no_op()
{
    let context = context().await;
    let mut t = Target::offscreen(&context, (4, 4), RGBA8);
    t.acquire(&context).unwrap().present(&context);
    t.acquire(&context).unwrap().present(&context);
}



#[tokio::test]
async fn depth_format_constant_matches_the_texture()
{
    let context = context().await;
    let t = Target::offscreen(&context, (4, 4), RGBA8);
    assert_eq!(Texture::DEPTH_FORMAT, wgpu::TextureFormat::Depth32Float);
    assert_eq!(t.depth().unwrap().texture.format(), Texture::DEPTH_FORMAT);
    assert_eq!(
        scig::core::pipeline::DepthDesc::reversed_z().format(),
        Texture::DEPTH_FORMAT
    );
}



#[tokio::test]
async fn zero_sized_textures_are_clamped_to_one_pixel()
{
    let context = context().await;
    let t = Target::offscreen(&context, (0, 0), RGBA8);
    assert_eq!(t.color().unwrap().size(), (1, 1));
}
