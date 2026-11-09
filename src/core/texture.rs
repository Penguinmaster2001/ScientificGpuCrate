use crate::core::context::Context;
use image::GenericImageView;
use std::assert_eq;



pub struct Texture
{
    #[allow(unused)]
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
}



impl Texture
{
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;



    pub fn from_bytes(context: &Context, bytes: &[u8]) -> anyhow::Result<Self>
    {
        let img = image::load_from_memory(bytes)?;
        Self::from_image(context, &img)
    }



    pub fn from_image(context: &Context, img: &image::DynamicImage) -> anyhow::Result<Self>
    {
        let rgba = img.to_rgba8();
        let dimensions = img.dimensions();

        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };
        let texture = context.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        context.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Ok(Self { texture, view })
    }



    pub fn create_depth_texture(context: &Context, size: impl Into<(u32, u32)>) -> Self
    {
        let size = size.into();
        let size = wgpu::Extent3d {
            width: size.0.max(1),
            height: size.1.max(1),
            depth_or_array_layers: 1,
        };
        let desc = wgpu::TextureDescriptor {
            label: None,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let texture = context.device.create_texture(&desc);

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self { texture, view }
    }



    pub fn create_color_target(
        context: &Context,
        size: impl Into<(u32, u32)>,
        format: wgpu::TextureFormat,
    ) -> Self
    {
        let size = size.into();
        let size = wgpu::Extent3d {
            width: size.0.max(1),
            height: size.1.max(1),
            depth_or_array_layers: 1,
        };
        let desc = wgpu::TextureDescriptor {
            label: None,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let texture = context.device.create_texture(&desc);

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self { texture, view }
    }



    pub fn size(&self) -> (u32, u32)
    {
        (self.texture.width(), self.texture.height())
    }
}



pub fn texture_data_to_image(
    data: &[u8],
    size: (u32, u32),
) -> image::ImageBuffer<image::Rgba<u8>, Vec<u8>>
{
    assert_eq!(size.0 * size.1 * 4, data.len() as _);

    let mut image = image::RgbaImage::new(size.0, size.1);
    for y in 0..size.1
    {
        for x in 0..size.0
        {
            let offset = ((y * size.0 * 4) + (x * 4)) as usize;
            let pixel = image::Rgba([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]);
            image.put_pixel(x, y, pixel);
        }
    }

    image
}
