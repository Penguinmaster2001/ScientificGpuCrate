use parameterized::parameterized;
use scig::core::{
    buffer::Buffer,
    context::{
        Context,
        ContextBuilder,
    },
    readback::{
        EncoderToken,
        ReadBack,
        TextureReadBack,
    },
    target::Target,
};



#[tokio::test]
async fn compute_buffer_readback() -> anyhow::Result<()>
{
    let context_builder = ContextBuilder::default();
    let context = Context::headless(context_builder).await?;
    let shader = context
        .device
        .create_shader_module(wgpu::include_wgsl!("shaders/double_buffer.wgsl"));

    let pipeline = context
        .device
        .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: None,
            module: &shader,
            entry_point: None,
            compilation_options: Default::default(),
            cache: Default::default(),
        });

    let input_data = (0..100).collect::<Vec<u32>>();

    let input_buffer = Buffer::from_slice(
        &context,
        wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
        &input_data,
    );

    let output_buffer = Buffer::<u32>::new(
        &context,
        wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::STORAGE,
        input_buffer.len(),
    );

    let bind_group = context
        .device
        .create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: input_buffer.handle().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: output_buffer.handle().as_entire_binding(),
                },
            ],
        });

    let mut encoder = context.device.create_command_encoder(&Default::default());

    {
        let num_dispatches = input_data.len().div_ceil(64) as u32;

        let mut pass = encoder.begin_compute_pass(&Default::default());
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups(num_dispatches, 1, 1);
    }

    let readback = ReadBack::encode(&context, &mut encoder, &output_buffer);
    context.queue.submit([encoder.finish()]);

    let output_data = readback.map(&context, EncoderToken { id: 0 })?;
    let expected_data = input_data.iter().map(|u| 2 * u).collect::<Vec<u32>>();
    assert_eq!(&expected_data, &output_data);

    Ok(())
}



#[tokio::test]
async fn context_buffer_compute() -> anyhow::Result<()>
{
    let context_builder = ContextBuilder::default();
    let context = Context::headless(context_builder).await?;
    let shader = context
        .device
        .create_shader_module(wgpu::include_wgsl!("shaders/double_buffer.wgsl"));

    let pipeline = context
        .device
        .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: None,
            module: &shader,
            entry_point: None,
            compilation_options: Default::default(),
            cache: Default::default(),
        });

    let input_data = (0..100).collect::<Vec<u32>>();

    let input_buffer = Buffer::from_slice(
        &context,
        wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
        &input_data,
    );

    let output_buffer = Buffer::<u32>::new(
        &context,
        wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::STORAGE,
        input_buffer.len(),
    );

    let temp_buffer = Buffer::<u32>::new(
        &context,
        wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        input_buffer.len(),
    );

    let bind_group = context
        .device
        .create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: input_buffer.handle().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: output_buffer.handle().as_entire_binding(),
                },
            ],
        });

    let mut encoder = context.device.create_command_encoder(&Default::default());

    {
        let num_dispatches = input_data.len().div_ceil(64) as u32;

        let mut pass = encoder.begin_compute_pass(&Default::default());
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups(num_dispatches, 1, 1);
    }

    encoder.copy_buffer_to_buffer(
        output_buffer.handle(),
        0,
        temp_buffer.handle(),
        0,
        output_buffer.handle().size(),
    );

    context.queue.submit([encoder.finish()]);

    let output_data = {
        let (tx, rx) = std::sync::mpsc::channel();

        temp_buffer
            .handle()
            .map_async(wgpu::MapMode::Read, .., move |result| {
                tx.send(result).unwrap()
            });

        context.device.poll(wgpu::PollType::wait_indefinitely())?;

        rx.recv()??;

        bytemuck::try_cast_slice(&temp_buffer.handle().get_mapped_range(..)?)
            .expect("Should be able to cast")
            .to_vec()
    };

    temp_buffer.handle().unmap();

    let expected_data = input_data.iter().map(|u| 2 * u).collect::<Vec<u32>>();
    assert_eq!(&expected_data, &output_data);

    Ok(())
}



#[parameterized(size = { (1, 1), (4, 4), (5, 3), (2, 7), (257, 1), (3, 384) })]
#[parameterized_macro(tokio::test)]
async fn texture_size_readback(size: (u32, u32)) -> anyhow::Result<()>
{
    let context_builder = ContextBuilder::default();
    let context = Context::headless(context_builder).await?;
    let format = wgpu::TextureFormat::Rgba8Unorm;
    let mut target = Target::offscreen(&context, size, format);
    let frame = target.acquire(&context)?;

    let mut encoder = context.device.create_command_encoder(&Default::default());

    {
        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &frame.color,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0,
                        g: 1.0,
                        b: 1.0,
                        a: 0.5,
                    }),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &frame.depth,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });
    }

    let readback = TextureReadBack::encode_texture(&context, &mut encoder, target.color().unwrap());

    context.queue.submit([encoder.finish()]);

    let output = readback.map_texture(&context, EncoderToken { id: 0 })?;
    let expected = [0u8, 255, 255, 128]
        .iter()
        .cycle()
        .take((format.block_copy_size(None).unwrap() * size.0 * size.1) as _)
        .copied()
        .collect::<Vec<u8>>();

    assert_eq!(output, expected,);

    Ok(())
}
