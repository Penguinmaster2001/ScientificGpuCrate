mod common;



use common::*;
use scig::core::{
    bind::BindGroupBuilder,
    buffer::{
        Buffer,
        BufferAlignment,
        GrowBuffer,
        PingPong,
    },
    pipeline::ComputePipelineDesc,
};



const STORE: wgpu::BufferUsages = wgpu::BufferUsages::STORAGE
    .union(wgpu::BufferUsages::COPY_SRC)
    .union(wgpu::BufferUsages::COPY_DST);



// ---------- Buffer<T> ----------

#[tokio::test]
async fn len_and_size_bytes_use_element_units()
{
    let context = context().await;
    let b = Buffer::<[f32; 4]>::new(&context, STORE, 10, BufferAlignment::None);
    assert_eq!(b.count(), 10);
    assert_eq!(b.size_bytes(), 160);
    let c = Buffer::from_slice(&context, STORE, &[1u32, 2, 3], BufferAlignment::None);
    assert_eq!(c.count(), 3);
    assert_eq!(c.size_bytes(), 12);
}



#[tokio::test]
async fn new_buffer_is_zeroed()
{
    let context = context().await;
    let b = Buffer::<u32>::new(&context, STORE, 16, BufferAlignment::None);
    assert_eq!(read_buffer(&context, &b), vec![0u32; 16]);
}



#[tokio::test]
async fn write_offset_is_in_elements()
{
    let context = context().await;
    let b = Buffer::from_slice(&context, STORE, &[0u32; 8], BufferAlignment::None);
    b.write(&context, 2, &[7, 8]);
    assert_eq!(read_buffer(&context, &b), vec![0, 0, 7, 8, 0, 0, 0, 0]);
    b.write(&context, 6, &[9, 9]);
    assert_eq!(read_buffer(&context, &b), vec![0, 0, 7, 8, 0, 0, 9, 9]);
}



#[cfg(debug_assertions)]
#[tokio::test]
#[should_panic]
async fn write_past_the_end_panics_in_debug()
{
    let context = context().await;
    let b = Buffer::from_slice(&context, STORE, &[0u32; 4], BufferAlignment::None);
    b.write(&context, 3, &[1, 2]);
}



#[tokio::test]
async fn odd_byte_length_round_trips()
{
    // copy_buffer_to_buffer needs multiples of 4. The wrapper must hide that.
    let context = context().await;
    let b = Buffer::from_slice(&context, STORE, &[1u8, 2, 3, 4, 5], BufferAlignment::None);
    assert_eq!(b.count(), 5);
    assert_eq!(read_buffer(&context, &b), vec![1u8, 2, 3, 4, 5]);
    let n = Buffer::<u8>::new(&context, STORE, 6, BufferAlignment::None);
    assert_eq!(read_buffer(&context, &n).len(), 6);
}



// ---------- GrowBuffer ----------


#[tokio::test]
async fn grow_buffer_writes_within_capacity_without_reallocating()
{
    let context = context().await;
    let mut g = GrowBuffer::<u32>::new(&context, STORE, 8, BufferAlignment::None);
    let gen0 = g.generation();
    assert!(!g.write(&context, &[1, 2, 3]));
    assert_eq!(g.generation(), gen0);
    assert_eq!(g.count(), 3);
    assert!(g.capacity() >= 8);
    assert_eq!(&read_buffer(&context, g.buffer())[..3], &[1, 2, 3]);
}



#[tokio::test]
async fn grow_buffer_reallocates_when_full_and_bumps_generation()
{
    let context = context().await;
    let mut g = GrowBuffer::<u32>::new(&context, STORE, 2, BufferAlignment::None);
    let gen0 = g.generation();
    let data: Vec<u32> = (0..100).collect();
    assert!(g.write(&context, &data));
    assert_eq!(g.generation(), gen0 + 1);
    assert!(g.capacity() >= 100);
    assert_eq!(g.count(), 100);
    assert_eq!(&read_buffer(&context, g.buffer())[..100], &data[..]);
}



#[tokio::test]
async fn grow_buffer_growth_is_geometric_and_never_shrinks()
{
    let context = context().await;
    let mut g = GrowBuffer::<u32>::new(&context, STORE, 1, BufferAlignment::None);
    let gen0 = g.generation();
    for n in 1..=1000usize
    {
        g.write(&context, &vec![n as u32; n]);
    }
    assert!(
        g.generation() - gen0 <= 12,
        "reallocated {} times",
        g.generation() - gen0
    );
    let cap = g.capacity();
    g.write(&context, &[1]);
    assert_eq!(g.capacity(), cap);
    assert_eq!(g.count(), 1);
}



#[tokio::test]
async fn grow_buffer_accepts_empty_writes()
{
    let context = context().await;
    let mut g = GrowBuffer::<u32>::new(&context, STORE, 4, BufferAlignment::None);
    assert!(!g.write(&context, &[]));
    assert_eq!(g.count(), 0);
}



#[tokio::test]
async fn grow_buffer_keeps_usage_flags_after_growth()
{
    // read_buffer needs COPY_SRC. This fails if reallocation drops flags.
    let context = context().await;
    let mut g = GrowBuffer::<u32>::new(&context, STORE, 1, BufferAlignment::None);
    g.write(&context, &[5; 64]);
    assert_eq!(read_buffer(&context, g.buffer())[63], 5);
}



// ---------- PingPong ----------

#[tokio::test]
async fn ping_pong_swaps_roles()
{
    let context = context().await;
    let a = Buffer::from_slice(&context, STORE, &[1u32], BufferAlignment::None);
    let b = Buffer::from_slice(&context, STORE, &[2u32], BufferAlignment::None);
    let mut pp = PingPong::new(a, b);
    assert_eq!(read_buffer(&context, pp.src()), [1]);
    assert_eq!(read_buffer(&context, pp.dst()), [2]);
    pp.flip();
    assert_eq!(read_buffer(&context, pp.src()), [2]);
    assert_eq!(read_buffer(&context, pp.dst()), [1]);
    pp.flip();
    assert_eq!(read_buffer(&context, pp.src()), [1]);
}



#[tokio::test]
async fn ping_pong_two_compute_steps()
{
    let mut h = Harness::new().await;
    let id = h
        .shaders
        .load_shader_source(&h.context, include_str!("shaders/double_buffer.wgsl"))
        .id();
    let pipeline = ComputePipelineDesc::new(id)
        .create_pipeline(&h.context, &h.shaders)
        .unwrap();
    let input: Vec<u32> = (0..100).collect();
    let a = Buffer::from_slice(&h.context, STORE, &input, BufferAlignment::None);
    let b = Buffer::<u32>::new(&h.context, STORE, 100, BufferAlignment::None);
    let mut pp = PingPong::new(a, b);

    for _ in 0..2
    {
        let bg = BindGroupBuilder::new()
            .add_entry(pp.src())
            .add_entry(pp.dst())
            .build_compute(&h.context, &pipeline);
        let mut enc = h.context.device.create_command_encoder(&Default::default());
        {
            let mut pass = enc.begin_compute_pass(&Default::default());
            pass.set_pipeline(&pipeline);
            pass.set_bind_group(0, &bg, &[]);
            pass.dispatch_workgroups(100u32.div_ceil(64), 1, 1);
        }
        h.context.queue.submit([enc.finish()]);
        pp.flip();
    }
    let expect: Vec<u32> = input.iter().map(|v| v * 4).collect();
    assert_eq!(read_buffer(&h.context, pp.src()), expect);
}
