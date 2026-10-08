use scig::core::{
    buffer::Buffer,
    context::{
        Context,
        ContextBuilder,
    },
};


#[tokio::test]
async fn headless_compute() -> anyhow::Result<()>
{
    let context_builder = ContextBuilder::default();
    let context = Context::headless(context_builder).await?;
    let shader = context
        .device
        .create_shader_module(wgpu::include_wgsl!("shaders/double_buffer.wgsl"));

    let pipeline = context
        .device
        .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Introduction Compute Pipeline"),
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


    {
        let (tx, rx) = std::sync::mpsc::channel();

        temp_buffer
            .handle()
            .map_async(wgpu::MapMode::Read, .., move |result| {
                tx.send(result).unwrap()
            });

        context.device.poll(wgpu::PollType::wait_indefinitely())?;

        rx.recv()??;

        let output_data = temp_buffer.handle().get_mapped_range(..)?;
        let expected_data = input_data.iter().map(|u| 2 * u).collect::<Vec<u32>>();
        assert_eq!(&expected_data, bytemuck::cast_slice(&output_data));
    }

    temp_buffer.handle().unmap();

    Ok(())
}
