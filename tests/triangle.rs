use scig::core::{
    context::{
        Context,
        ContextBuilder,
    },
    pass::{
        PassOps,
        begin_render_pass,
    },
    pipeline::{
        DepthDesc,
        PipelineCache,
        RenderPipelineDesc,
    },
    readback::{
        EncoderToken,
        TextureReadBack,
    },
    shader::ShaderLib,
    target::Target,
};



const W: u32 = 8;
const H: u32 = 8;



async fn render(
    context: &Context,
    format: wgpu::TextureFormat,
    fs: &'static str,
    front_face: wgpu::FrontFace,
    depth_clear: f32,
) -> anyhow::Result<Vec<u8>>
{
    let mut shaders = ShaderLib::new();
    let shader = shaders
        .load_shader_source(context, include_str!("shaders/triangle.wgsl"))
        .id();

    let desc = RenderPipelineDesc::new(shader, "vs", Some(fs))
        .color_target(format, None)
        .depth(DepthDesc::reversed_z()) // DEPTH_FORMAT, Greater, write on
        .front_face(front_face); // default cull: Back
    let mut cache = PipelineCache::new();
    let pipeline = cache.render(context, &shaders, &desc)?; // Arc<wgpu::RenderPipeline>

    let mut target = Target::offscreen(context, (W, H), format);
    let frame = target.acquire(context)?;
    let mut encoder = context.device.create_command_encoder(&Default::default());
    {
        let mut pass = begin_render_pass(
            &mut encoder,
            &frame,
            PassOps::clear(wgpu::Color::BLACK, depth_clear),
        );
        pass.set_pipeline(&pipeline);
        pass.draw(0..3, 0..1);
    }

    let rb = TextureReadBack::encode_texture(context, &mut encoder, target.color().unwrap());
    context.queue.submit([encoder.finish()]);
    rb.map_texture(context, EncoderToken { id: 0 })
}



fn px(d: &[u8], x: u32, y: u32) -> [u8; 4]
{
    let i = ((y * W + x) * 4) as usize;
    d[i..i + 4].try_into().unwrap()
}


const RED: [u8; 4] = [255, 0, 0, 255];
const CLEAR: [u8; 4] = [0, 0, 0, 255];



#[tokio::test]
async fn orientation_and_winding() -> anyhow::Result<()>
{
    let ctx = Context::headless(ContextBuilder::default()).await?;
    let f = wgpu::TextureFormat::Rgba8Unorm;
    let out = render(&ctx, f, "fs", wgpu::FrontFace::Ccw, 0.0).await?;


    assert_eq!(px(&out, 0, H - 1), RED); // bottom-left: inside
    assert_eq!(px(&out, W - 1, 0), CLEAR); // top-right: outside
    let out = render(&ctx, f, "fs", wgpu::FrontFace::Cw, 0.0).await?;
    assert_eq!(px(&out, 0, H - 1), CLEAR); // now back-facing: culled
    Ok(())
}



#[tokio::test]
async fn depth_greater_rejects() -> anyhow::Result<()>
{
    let ctx = Context::headless(ContextBuilder::default()).await?;
    let f = wgpu::TextureFormat::Rgba8Unorm;
    let out = render(&ctx, f, "fs", wgpu::FrontFace::Ccw, 1.0).await?; // 0.5 > 1.0 is false
    assert_eq!(px(&out, 0, H - 1), CLEAR);
    Ok(())
}



#[tokio::test]
async fn srgb_encoding() -> anyhow::Result<()>
{
    let context = Context::headless(ContextBuilder::default()).await?;
    let lin = render(
        &context,
        wgpu::TextureFormat::Rgba8Unorm,
        "fs_half",
        wgpu::FrontFace::Ccw,
        0.0,
    )
    .await?;
    let srgb = render(
        &context,
        wgpu::TextureFormat::Rgba8UnormSrgb,
        "fs_half",
        wgpu::FrontFace::Ccw,
        0.0,
    )
    .await?;
    assert!(px(&lin, 0, H - 1)[0].abs_diff(128) <= 1);
    assert!(px(&srgb, 0, H - 1)[0].abs_diff(188) <= 1);
    Ok(())
}
