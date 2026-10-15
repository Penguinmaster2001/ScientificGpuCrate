use crate::core::{
    buffer::Buffer,
    context::Context,
    texture::Texture,
};
use wgpu::TexelCopyBufferLayout;



#[derive(PartialEq, Eq, Debug)]
pub struct EncoderToken
{
    pub id: u32,
}



pub struct ReadBack<T>
{
    staging_buffer: Buffer<T>,
    token: EncoderToken,
}



impl<T: bytemuck::Pod> ReadBack<T>
{
    pub fn encode(context: &Context, encoder: &mut wgpu::CommandEncoder, data: &Buffer<T>) -> Self
    {
        let staging_buffer = Buffer::new(
            context,
            wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            data.len(),
        );

        encoder.copy_buffer_to_buffer(
            data.handle(),
            0,
            staging_buffer.handle(),
            0,
            data.handle().size(),
        );

        Self {
            staging_buffer,
            token: EncoderToken { id: 0 },
        }
    }



    pub fn map(self, context: &Context, token: EncoderToken) -> anyhow::Result<Vec<T>>
    {
        debug_assert_eq!(self.token, token);

        let output_data = {
            let (tx, rx) = std::sync::mpsc::channel();

            self.staging_buffer
                .handle()
                .map_async(wgpu::MapMode::Read, .., move |result| {
                    tx.send(result).unwrap()
                });

            context.device.poll(wgpu::PollType::wait_indefinitely())?;

            rx.recv()??;

            bytemuck::try_cast_slice(&self.staging_buffer.handle().get_mapped_range(..)?)
                .expect("Should be able to cast")
                .to_vec()
        };

        self.staging_buffer.handle().unmap();

        Ok(output_data)
    }
}



#[derive(Debug)]
pub struct TextureReadBack
{
    staging_buffer: Buffer<u8>,
    texel_copy_size_bytes: u32,
    texture_size_pixels: (u32, u32),
    padded_size_bytes: u32,
    token: EncoderToken,
}



impl TextureReadBack
{
    fn new(context: &Context, texture: &Texture) -> Self
    {
        let texel_copy_size_bytes = texture
            .texture
            .format()
            .block_copy_size(Some(wgpu::TextureAspect::All))
            .unwrap();
        let texture_size_pixels = texture.size();
        let unpadded = texture_size_pixels.0 * texel_copy_size_bytes;
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let padded = unpadded.div_ceil(align) * align;
        let buffer_size = padded * texture_size_pixels.1;

        let staging_buffer = Buffer::new(
            context,
            wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            buffer_size as _,
        );

        Self {
            staging_buffer,
            texel_copy_size_bytes,
            texture_size_pixels,
            padded_size_bytes: padded,
            token: EncoderToken { id: 0 },
        }
    }



    pub fn encode_texture(
        context: &Context,
        encoder: &mut wgpu::CommandEncoder,
        texture: &Texture,
    ) -> Self
    {
        let rb = Self::new(context, texture);

        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                aspect: wgpu::TextureAspect::All,
                texture: &texture.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: rb.staging_buffer.handle(),
                layout: TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(rb.padded_size_bytes),
                    rows_per_image: Some(rb.texture_size_pixels.1),
                },
            },
            texture.texture.size(),
        );

        rb
    }



    pub fn map_texture(self, context: &Context, token: EncoderToken) -> anyhow::Result<Vec<u8>>
    {
        debug_assert_eq!(self.token, token);

        let output_data = {
            let (tx, rx) = std::sync::mpsc::channel();

            self.staging_buffer
                .handle()
                .map_async(wgpu::MapMode::Read, .., move |result| {
                    tx.send(result).unwrap()
                });

            context.device.poll(wgpu::PollType::wait_indefinitely())?;

            rx.recv()??;

            self.staging_buffer
                .handle()
                .get_mapped_range(..)?
                .chunks_exact(self.padded_size_bytes as _) // chunk is padded byte row
                .take(self.texture_size_pixels.1 as _) // Truncate to row count
                .flat_map(|c| {
                    c.iter()
                        .take((self.texture_size_pixels.0 * self.texel_copy_size_bytes) as _)
                })
                .copied()
                .collect()
        };

        self.staging_buffer.handle().unmap();

        Ok(output_data)
    }
}
