use crate::core::{
    context::Context,
    texture::Texture,
};
use std::{
    fmt::Display,
    sync::Arc,
};
use wgpu::{
    CurrentSurfaceTexture::{
        self,
        Success,
    },
    SurfaceTexture,
};
use winit::window::Window;



pub enum Target
{
    Surface
    {
        surface: wgpu::Surface<'static>,
        config: wgpu::SurfaceConfiguration,
    },
    Offscreen
    {
        color: Texture,
        depth: Texture,
        format: wgpu::TextureFormat,
        generation: u64,
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
        Self::Offscreen {
            color: Texture::create_color_target(context, size, format),
            depth: Texture::create_depth_texture(context, size),
            generation: 0,
            format,
        }
    }



    pub fn window(context: &Context, window: Arc<Window>) -> anyhow::Result<Self>
    {
        let surface = context.instance.create_surface(window.clone())?;

        let surface_caps = surface.get_capabilities(&context.adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let size = window.inner_size();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
            color_space: wgpu::SurfaceColorSpace::Auto,
        };

        Ok(Self::Surface { surface, config })
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
            Target::Surface { surface, config } =>
            {
                config.width = size.0;
                config.height = size.1;
                surface.configure(&context.device, config);
            }
            Target::Offscreen {
                color,
                depth,
                format,
                generation,
            } =>
            {
                if size == color.size()
                {
                    return;
                }
                *color = Texture::create_color_target(context, size, *format);
                *depth = Texture::create_depth_texture(context, size);
                *generation += 1;
            }
        }
    }



    #[expect(unused_variables, unreachable_code)]
    pub fn acquire(&mut self, context: &Context) -> anyhow::Result<Frame, FrameError>
    {
        match self
        {
            Target::Surface { surface, config } =>
            {
                let tex = get_surface_texture(surface)?;

                Ok(Frame {
                    color: todo!(),
                    depth: todo!(),
                    size: [config.width, config.height],
                    format: config.format,
                    present_handle: Some(tex),
                })
            }
            Target::Offscreen {
                color,
                depth,
                format,
                generation: _,
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
            Target::Surface {
                surface: _,
                config: _,
            } => None,
            Target::Offscreen {
                color,
                depth: _,
                format: _,
                generation: _,
            } => Some(color),
        }
    }



    pub fn depth(&self) -> Option<&Texture>
    {
        match self
        {
            Target::Surface {
                surface: _,
                config: _,
            } => None,
            Target::Offscreen {
                color: _,
                depth,
                format: _,
                generation: _,
            } => Some(depth),
        }
    }



    pub fn generation(&self) -> u64
    {
        match self
        {
            Target::Surface {
                surface: _,
                config: _,
            } => 0,
            Target::Offscreen {
                color: _,
                depth: _,
                format: _,
                generation,
            } => *generation,
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
    Suboptimal,
    Timeout,
    Occluded,
    Outdated,
    Lost,
    Validation,
}



fn get_surface_texture(surface: &wgpu::Surface<'static>) -> Result<SurfaceTexture, FrameError>
{
    match surface.get_current_texture()
    {
        Success(surface_texture) => Ok(surface_texture),
        CurrentSurfaceTexture::Suboptimal(_) => Err(FrameError::Suboptimal),
        CurrentSurfaceTexture::Timeout => Err(FrameError::Timeout),
        CurrentSurfaceTexture::Occluded => Err(FrameError::Occluded),
        CurrentSurfaceTexture::Outdated => Err(FrameError::Outdated),
        CurrentSurfaceTexture::Lost => Err(FrameError::Lost),
        CurrentSurfaceTexture::Validation => Err(FrameError::Validation),
    }
}



impl Display for FrameError
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        std::write!(f, "{:?}", self)
    }
}



impl std::error::Error for FrameError {}
