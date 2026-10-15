use crate::core::{
    context::Context,
    texture::Texture,
};
use std::fmt::Display;



#[expect(clippy::large_enum_variant)]
pub enum Target
{
    Surface,
    Offscreen
    {
        color: Texture,
        depth: Texture,
        format: wgpu::TextureFormat,
    },
}



impl Target
{
    pub fn offscreen(
        context: &Context,
        size: impl Into<(u32, u32)>,
        format: wgpu::TextureFormat,
    ) -> Self
    {
        let size = size.into();
        Target::Offscreen {
            color: Texture::create_color_target(context, size, format),
            depth: Texture::create_depth_texture(context, size),
            format,
        }
    }



    pub fn resize(&mut self, context: &Context, size: impl Into<(u32, u32)>)
    {
        let size = size.into();
        if size.0 == 0 || size.1 == 0
        {
            return;
        }

        match self
        {
            Target::Surface => todo!(),
            Target::Offscreen {
                color,
                depth,
                format,
            } =>
            {
                if size == color.size()
                {
                    return;
                }
                *color = Texture::create_color_target(context, size, *format);
                *depth = Texture::create_depth_texture(context, size);
            }
        }
    }



    pub fn acquire(&mut self, _context: &Context) -> anyhow::Result<Frame, FrameError>
    {
        match self
        {
            Target::Surface => todo!(),
            Target::Offscreen {
                color,
                depth,
                format,
            } => Ok(Frame {
                color: color.view.clone(),
                depth: depth.view.clone(),
                size: depth.size().into(),
                format: *format,
                present_handle: None,
            }),
        }
    }


    pub fn color(&self) -> Option<&Texture>
    {
        match self
        {
            Target::Surface => None,
            Target::Offscreen {
                color,
                depth: _,
                format: _,
            } => Some(color),
        }
    }
}



pub struct Frame
{
    pub color: wgpu::TextureView,
    pub depth: wgpu::TextureView,
    pub size: [u32; 2],
    pub format: wgpu::TextureFormat,
    present_handle: Option<wgpu::SurfaceTexture>,
}



impl Frame
{
    pub fn present(self, context: &Context)
    {
        if let Some(surface_texture) = self.present_handle
        {
            context.queue.present(surface_texture);
        }
    }
}



#[derive(Debug)]
pub enum FrameError
{
    Lost,
    Outdated,
    Timeout,
    OutOfMemory,
}



impl Display for FrameError
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        write!(f, "{:?}", self)
    }
}



impl std::error::Error for FrameError {}
