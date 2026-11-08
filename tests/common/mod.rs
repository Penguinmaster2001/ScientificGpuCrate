#![allow(dead_code)]



use bytemuck::Pod;
use scig::core::{
    buffer::Buffer,
    context::{
        Context,
        ContextBuilder,
    },
    math::{
        self,
        Mat4,
    },
    pass::{
        FrameContext,
        FrameTime,
        Pass,
        PassOps,
        begin_render_pass,
        encode_frame,
    },
    pipeline::{
        DepthDesc,
        PipelineCache,
        RenderPipelineDesc,
    },
    readback::{
        EncoderToken,
        ReadBack,
        TextureReadBack,
    },
    shader::ShaderLib,
    target::Target,
    uniform::Uniform,
    view::{
        View,
        ViewBuffer,
        ViewProjection,
    },
};



pub const BLACK: [u8; 4] = [0, 0, 0, 255];
pub const WHITE: [u8; 4] = [255, 255, 255, 255];
pub const RED: [u8; 4] = [255, 0, 0, 255];
pub const SOLID: &str = include_str!("../shaders/solid.wgsl");
pub const RGBA8: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;



pub async fn context() -> Context
{
    Context::headless(ContextBuilder::default())
        .await
        .expect("no adapter")
}



// ---------- pixel helpers ----------



pub fn px(d: &[u8], w: u32, x: u32, y: u32) -> [u8; 4]
{
    let i = ((y * w + x) * 4) as usize;
    d[i..i + 4].try_into().unwrap()
}



pub fn near(a: [u8; 4], b: [u8; 4], tol: u8) -> bool
{
    a.iter().zip(b.iter()).all(|(x, y)| x.abs_diff(*y) <= tol)
}



pub fn pixels(d: &[u8], w: u32) -> impl Iterator<Item = (u32, u32, [u8; 4])> + '_
{
    d.chunks_exact(4).enumerate().map(move |(i, c)| {
        let i = i as u32;
        (i % w, i / w, [c[0], c[1], c[2], c[3]])
    })
}



pub fn is_black(p: [u8; 4]) -> bool
{
    p[0] == 0 && p[1] == 0 && p[2] == 0
}



pub fn lit_count(d: &[u8], w: u32) -> usize
{
    pixels(d, w).filter(|(_, _, p)| !is_black(*p)).count()
}



pub fn find_color(d: &[u8], w: u32, c: [u8; 4]) -> Option<(u32, u32)>
{
    pixels(d, w)
        .find(|(_, _, p)| *p == c)
        .map(|(x, y, _)| (x, y))
}



pub fn centroid_x(d: &[u8], w: u32, pred: impl Fn([u8; 4]) -> bool) -> Option<f32>
{
    let (sum, n) = pixels(d, w)
        .filter(|(_, _, p)| pred(*p))
        .fold((0.0f32, 0u32), |(s, n), (x, _, _)| (s + x as f32, n + 1));
    (n > 0).then(|| sum / n as f32)
}



pub fn runs_in_row(d: &[u8], w: u32, y: u32) -> usize
{
    let mut runs = 0;
    let mut inside = false;
    for x in 0..w
    {
        let lit = !is_black(px(d, w, x, y));
        if lit && !inside
        {
            runs += 1;
        }
        inside = lit;
    }
    runs
}



pub fn approx(a: f32, b: f32, eps: f32) -> bool
{
    (a - b).abs() <= eps
}



pub fn mat_approx(a: &Mat4, b: &Mat4, eps: f32) -> bool
{
    (0..4).all(|c| (0..4).all(|r| approx(a[c][r], b[c][r], eps)))
}



pub fn vec_approx<const N: usize>(a: [f32; N], b: [f32; N], eps: f32) -> bool
{
    a.iter().zip(b.iter()).all(|(x, y)| approx(*x, *y, eps))
}



// ---------- cameras for tests ----------



pub fn eye_view(x: f32) -> glam::Affine3
{
    math::look_at([x, 0.0, 0.0], [x, 0.0, -1.0], [0.0, 1.0, 0.0])
}



pub fn proj90() -> ViewProjection
{
    ViewProjection::PerspectiveInfRev { fov_deg: 90.0 }
}



pub fn cam_view() -> View
{
    View::new(glam::Affine3::IDENTITY, proj90())
}



pub fn dummy_view() -> View
{
    View::new(glam::Affine3::IDENTITY, math::identity())
}



// ---------- buffer helper ----------



pub fn read_buffer<T: Pod + Uniform>(ctx: &Context, b: &Buffer<T>) -> Vec<T>
{
    let mut enc = ctx.device.create_command_encoder(&Default::default());
    let rb = ReadBack::encode(ctx, &mut enc, b);
    ctx.queue.submit([enc.finish()]);
    rb.map(ctx, EncoderToken { id: 0 }).unwrap()
}



// ---------- offscreen frame harness ----------



pub struct Harness
{
    pub context: Context,
    pub shaders: ShaderLib,
    pub pipelines: PipelineCache,
    pub view_buffer: ViewBuffer,
}



impl Harness
{
    pub async fn new() -> Self
    {
        let context = context().await;
        let view_buffer = ViewBuffer::new(&context);
        Self {
            context,
            shaders: ShaderLib::new(),
            pipelines: PipelineCache::new(),
            view_buffer,
        }
    }



    pub fn render(
        &mut self,
        size: (u32, u32),
        format: wgpu::TextureFormat,
        clear: wgpu::Color,
        views: &[View],
        passes: &mut [&mut dyn Pass],
    ) -> Vec<u8>
    {
        let mut target = Target::offscreen(&self.context, size, format);
        let frame = target.acquire(&self.context).unwrap();
        let mut encoder = self
            .context
            .device
            .create_command_encoder(&Default::default());
        encode_frame(
            &self.context,
            &mut self.shaders,
            &mut self.pipelines,
            &mut self.view_buffer,
            &mut encoder,
            &frame,
            views,
            clear,
            passes,
            FrameTime::default(),
        );
        let rb =
            TextureReadBack::encode_texture(&self.context, &mut encoder, target.color().unwrap());
        self.context.queue.submit([encoder.finish()]);
        frame.present(&self.context);
        rb.map_texture(&self.context, EncoderToken { id: 0 })
            .unwrap()
    }



    pub fn render_rgba8(
        &mut self,
        size: (u32, u32),
        views: &[View],
        passes: &mut [&mut dyn Pass],
    ) -> Vec<u8>
    {
        self.render(size, RGBA8, wgpu::Color::BLACK, views, passes)
    }



    pub fn solid_desc(
        &mut self,
        fs: &'static str,
        format: wgpu::TextureFormat,
    ) -> RenderPipelineDesc
    {
        let shader = self.shaders.load_shader_source(&self.context, SOLID).id();
        RenderPipelineDesc::new(shader, "vs", Some(fs))
            .color_target(format, None)
            .depth(DepthDesc::reversed_z())
    }
}



pub struct TrianglePass
{
    pub desc: RenderPipelineDesc,
    pub instances: u32,
    pub uses_view: bool,
}



impl TrianglePass
{
    pub fn new(desc: RenderPipelineDesc) -> Self
    {
        Self {
            desc,
            instances: 1,
            uses_view: false,
        }
    }
}



impl Pass for TrianglePass
{
    fn draw(&mut self, context: &mut FrameContext<'_>, view: usize)
    {
        let desc = self.desc.clone().color_mask(context.views[view].color_mask);
        let pipeline = context
            .pipelines
            .render(context.context, context.shaders, &desc)
            .unwrap();

        let mut pass = begin_render_pass(context.encoder, context.frame, PassOps::load());
        pass.set_pipeline(&pipeline);

        if self.uses_view
        {
            pass.set_bind_group(
                0,
                context.view_buffer.bind_group(),
                &[context.view_buffer.offset(view)],
            );
        }
        pass.draw(0..3, 0..self.instances);
    }
}
