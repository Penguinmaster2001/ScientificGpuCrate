use crate::common::{
    Harness,
    approx,
    cam_view,
    eye_view,
    proj90,
    vec_approx,
};
use scig::core::{
    buffer::Buffer,
    math,
    readback::{
        EncoderToken,
        ReadBack,
    },
    view::{
        View,
        ViewProjection,
        ViewUniform,
    },
};



mod common;



const EPS: f32 = 1.0e-4;



// ---------- ViewUniform (CPU) ----------


#[test]
fn view_uniform_position_comes_from_inverse_view()
{
    let u = ViewUniform::new(
        &View::new(
            math::look_at([1.0, 2.0, 3.0], [9.0, 9.0, 9.0], [0.0, 1.0, 0.0]),
            ViewProjection::proj90(),
        ),
        [64, 64],
    );

    assert!(vec_approx(u.position, [1.0, 2.0, 3.0, 1.0], EPS));
}



#[test]
fn view_uniform_viewport_layout_and_zero_guard()
{
    let u = ViewUniform::new(&cam_view(), [640, 360]);
    assert!(vec_approx(
        u.viewport,
        [640.0, 360.0, 1.0 / 640.0, 1.0 / 360.0],
        EPS
    ));
    let z = ViewUniform::new(&cam_view(), [0, 0]);
    assert!(z.viewport.iter().all(|v| v.is_finite()));
}



#[test]
fn view_defaults_write_all_channels()
{
    assert_eq!(cam_view().color_mask, wgpu::ColorWrites::ALL);
    let v = cam_view().mask(wgpu::ColorWrites::RED);
    assert_eq!(v.color_mask, wgpu::ColorWrites::RED);
}



// ---------- ViewBuffer (GPU) ----------



fn uniform_at(x: f32) -> ViewUniform
{
    ViewUniform::new(&View::new(eye_view(x), proj90()), [64, 64])
}



// Binds slot `slot` with its dynamic offset in a compute shader and returns
// [position.x, viewport.x, view_proj[0][0], inv_view[3][0]].
fn read_slot(h: &Harness, slot: usize) -> Vec<f32>
{
    let context = &h.context;
    let module = context
        .device
        .create_shader_module(wgpu::include_wgsl!("shaders/read_view.wgsl"));
    let out_layout = context
        .device
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
    let layout = context
        .device
        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[Some(h.view_buffer.layout()), Some(&out_layout)],
            immediate_size: 0,
        });
    let pipeline = context
        .device
        .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: Some(&layout),
            module: &module,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: Default::default(),
        });
    let out = Buffer::<f32>::new(
        context,
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        4,
        false,
    );
    let out_group = context
        .device
        .create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &out_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: out.handle().as_entire_binding(),
            }],
        });
    let mut enc = context.device.create_command_encoder(&Default::default());
    {
        let mut pass = enc.begin_compute_pass(&Default::default());
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, h.view_buffer.bind_group(), &[h.view_buffer.offset(slot)]);
        pass.set_bind_group(1, &out_group, &[]);
        pass.dispatch_workgroups(1, 1, 1);
    }
    let rb = ReadBack::encode(context, &mut enc, &out);
    context.queue.submit([enc.finish()]);
    rb.map(context, EncoderToken { id: 0 }).unwrap()
}



#[tokio::test]
async fn stride_respects_device_alignment()
{
    let h = Harness::new().await;
    let align = h
        .context
        .device
        .limits()
        .min_uniform_buffer_offset_alignment as u64;
    let stride = h.view_buffer.stride();
    assert!(stride >= 368);
    assert_eq!(stride % align, 0);
    assert!(
        stride < 368 + align,
        "stride must be the smallest aligned size"
    );
    assert_eq!(h.view_buffer.offset(0), 0);
    assert_eq!(h.view_buffer.offset(3) as u64, 3 * stride);
}



#[tokio::test]
async fn each_slot_is_visible_through_its_own_dynamic_offset()
{
    // All slots are written before any submit. A single rewritten uniform would
    // show the last value in every read.
    let mut h = Harness::new().await;
    h.view_buffer.upload(
        &h.context,
        &[uniform_at(10.0), uniform_at(20.0), uniform_at(30.0)],
    );
    for (slot, x) in [(0usize, 10.0f32), (1, 20.0), (2, 30.0)]
    {
        let r = read_slot(&h, slot);
        println!(
            "{slot}, {x}\n{:?}\n\n{:?}\n\n\n\n",
            bytemuck::cast_slice::<u8, f32>(bytemuck::bytes_of(&r[0])),
            bytemuck::cast_slice::<u8, f32>(bytemuck::bytes_of(&uniform_at(x)))
        );
        assert!(
            vec_approx([r[0], r[1], r[2], r[3]], [x, 64.0, 1.0, x], 1.0e-3),
            "slot {slot}: {r:?}"
        );
    }
}



#[tokio::test]
async fn upload_grows_and_keeps_layout_stable()
{
    let mut h = Harness::new().await;
    h.view_buffer.upload(&h.context, &[uniform_at(1.0)]);
    let layout_before = h.view_buffer.layout().clone();
    let xs: Vec<f32> = (0..9).map(|i| i as f32 * 3.0).collect();
    let us: Vec<ViewUniform> = xs.iter().map(|x| uniform_at(*x)).collect();
    h.view_buffer.upload(&h.context, &us);
    assert!(
        h.view_buffer.layout().clone() == layout_before,
        "layout identity must not change"
    );
    for slot in [0usize, 4, 8]
    {
        let r = read_slot(&h, slot);
        println!("{slot}, {r:?}, {}", xs[slot]);
        assert!(approx(r[0], xs[slot], 1.0e-3), "slot {slot}: {r:?}");
    }
}



#[tokio::test]
async fn upload_empty_slice_is_ok()
{
    let mut h = Harness::new().await;
    h.view_buffer.upload(&h.context, &[]);
}



#[tokio::test]
async fn upload_replaces_previous_frame()
{
    let mut h = Harness::new().await;
    h.view_buffer
        .upload(&h.context, &[uniform_at(5.0), uniform_at(6.0)]);
    h.view_buffer.upload(&h.context, &[uniform_at(7.0)]);
    assert!(approx(read_slot(&h, 0)[0], 7.0, 1.0e-3));
}
