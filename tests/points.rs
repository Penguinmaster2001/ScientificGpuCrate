mod common;



use common::*;
use function_name::named;
use scig::{
    core::{
        bind::BindGroupBuilder,
        buffer::{
            Buffer,
            BufferAlignment,
        },
        pipeline::ComputePipelineDesc,
        texture::texture_data_to_image,
    },
    passes::points::{
        Point,
        PointBlend,
        PointPass,
        PointPassConfig,
        PointUniform,
    },
};



const N: u32 = 64;
const GREEN: [u8; 4] = [0, 255, 0, 255];
const BLUE: [u8; 4] = [0, 0, 255, 255];
const SAVE_IMAGES: bool = false;



macro_rules! draw_and_save {
    ($h:expr, $pass:expr) => {{
        let out = draw($h, $pass);
        let name = format!(
            "tests/results/{}_{}.png",
            module_path!().replace("::", "_"),
            function_name!()
        );

        if SAVE_IMAGES
        {
            texture_data_to_image(&out, (N, N)).save(name).unwrap();
        }

        out
    }};
    ($h:expr, $pass:expr, $label:expr) => {{
        let out = draw($h, $pass);
        let name = format!(
            "tests/results/{}_{}_{}.png",
            module_path!().replace("::", "_"),
            function_name!(),
            $label
        );

        if SAVE_IMAGES
        {
            texture_data_to_image(&out, (N, N)).save(name).unwrap();
        }

        out
    }};
}



fn pt(x: f32, y: f32, z: f32, size: f32, color: [f32; 4]) -> Point
{
    Point {
        pos: [x, y, z].into(),
        data: [size, 0.0, 0.0, 0.0],
        color,
    }
}



fn config(blend: PointBlend, depth: bool) -> PointPassConfig
{
    PointPassConfig {
        blend,
        depth,
        ..PointPassConfig::new(RGBA8)
    }
}



fn draw(h: &mut Harness, pass: &mut PointPass) -> Vec<u8>
{
    h.render_rgba8((N, N), &[cam_view()], &[pass])
}



#[test]
fn point_is_48_bytes_with_the_wgsl_layout()
{
    assert_eq!(std::mem::size_of::<PointUniform>(), 48);
    assert_eq!(std::mem::offset_of!(PointUniform, pos), 0);
    assert_eq!(std::mem::offset_of!(PointUniform, data), 16);
    assert_eq!(std::mem::offset_of!(PointUniform, color), 32);
}



#[named]
#[tokio::test]
async fn single_point_is_centered_and_world_sized()
{
    let mut h = Harness::new().await;
    let mut pass = PointPass::new(&h.context, &mut h.shaders, config(PointBlend::Opaque, true));
    // size 1 at distance 5: 1/5 of the half-screen (32 px) per unit gives about 6.4 px across.
    pass.set_points(&h.context, &[pt(0.0, 0.0, -5.0, 1.0, [1.0, 0.0, 0.0, 1.0])]);
    let out = draw_and_save!(&mut h, &mut pass);

    assert_eq!(px(&out, N, 32, 32), RED);
    assert_eq!(px(&out, N, 0, 0), BLACK);
    let n = lit_count(&out, N);
    assert!((20..=60).contains(&n), "{n} lit pixels");
}



#[named]
#[tokio::test]
async fn axes_are_x_right_and_y_up()
{
    let mut h = Harness::new().await;
    let mut pass = PointPass::new(&h.context, &mut h.shaders, config(PointBlend::Opaque, true));
    pass.set_points(
        &h.context,
        &[
            pt(-2.0, 0.0, -5.0, 1.0, [0.0, 1.0, 0.0, 1.0]),
            pt(2.0, 0.0, -5.0, 1.0, [0.0, 0.0, 1.0, 1.0]),
            pt(0.0, 2.0, -5.0, 1.0, [1.0, 0.0, 0.0, 1.0]),
        ],
    );
    let out = draw_and_save!(&mut h, &mut pass);

    let (gx, gy) = find_color(&out, N, GREEN).expect("green");
    let (bx, _) = find_color(&out, N, BLUE).expect("blue");
    let (_, ry) = find_color(&out, N, RED).expect("red");
    assert!(gx < 32 && bx > 32, "x must grow to the right");
    assert!(ry < gy, "y must grow upward (smaller row index)");
}



#[named]
#[tokio::test]
async fn size_is_in_world_units()
{
    let mut h = Harness::new().await;
    let mut pass = PointPass::new(&h.context, &mut h.shaders, config(PointBlend::Opaque, true));
    let white = [1.0; 4];
    pass.set_points(&h.context, &[pt(0.0, 0.0, -5.0, 2.0, white)]);
    let near_count = lit_count(&draw_and_save!(&mut h, &mut pass, "near_count"), N) as f32;
    pass.set_points(&h.context, &[pt(0.0, 0.0, -10.0, 2.0, white)]);
    let far_count = lit_count(&draw_and_save!(&mut h, &mut pass, "far_count"), N) as f32;
    let ratio = near_count / far_count;
    assert!(
        (3.0..=5.2).contains(&ratio),
        "area ratio {ratio} for 2x distance"
    );
}



#[named]
#[tokio::test]
async fn behind_the_camera_is_not_drawn()
{
    let mut h = Harness::new().await;
    let mut pass = PointPass::new(&h.context, &mut h.shaders, config(PointBlend::Opaque, true));
    pass.set_points(&h.context, &[pt(0.0, 0.0, 5.0, 2.0, [1.0; 4])]);
    assert_eq!(lit_count(&draw(&mut h, &mut pass), N), 0);
}



#[named]
#[tokio::test]
async fn depth_test_orders_points_regardless_of_buffer_order()
{
    let mut h = Harness::new().await;
    let mut pass = PointPass::new(&h.context, &mut h.shaders, config(PointBlend::Opaque, true));
    let near_red = pt(0.0, 0.0, -4.0, 2.0, [1.0, 0.0, 0.0, 1.0]);
    let far_green = pt(0.0, 0.0, -8.0, 2.0, [0.0, 1.0, 0.0, 1.0]);
    for order in [[near_red, far_green], [far_green, near_red]]
    {
        pass.set_points(&h.context, &order);
        assert_eq!(px(&draw_and_save!(&mut h, &mut pass), N, 32, 32), RED);
    }
}



#[named]
#[tokio::test]
async fn without_depth_the_last_point_wins()
{
    let mut h = Harness::new().await;
    let mut pass = PointPass::new(
        &h.context,
        &mut h.shaders,
        config(PointBlend::Opaque, false),
    );
    let near_red = pt(0.0, 0.0, -4.0, 2.0, [1.0, 0.0, 0.0, 1.0]);
    let far_green = pt(0.0, 0.0, -8.0, 2.0, [0.0, 1.0, 0.0, 1.0]);
    pass.set_points(&h.context, &[near_red, far_green]);
    assert_eq!(
        px(
            &draw_and_save!(&mut h, &mut pass, "near_red_far_green"),
            N,
            32,
            32
        ),
        GREEN
    );
    pass.set_points(&h.context, &[far_green, near_red]);
    assert_eq!(
        px(
            &draw_and_save!(&mut h, &mut pass, "far_green_near_red"),
            N,
            32,
            32
        ),
        RED
    );
}



#[named]
#[tokio::test]
async fn additive_blend_sums_overlapping_points()
{
    let mut h = Harness::new().await;
    let half = pt(0.0, 0.0, -5.0, 2.0, [0.5, 0.0, 0.0, 1.0]);

    let mut add = PointPass::new(
        &h.context,
        &mut h.shaders,
        config(PointBlend::Additive, false),
    );
    add.set_points(&h.context, &[half, half]);
    assert!(px(&draw_and_save!(&mut h, &mut add, "add"), N, 32, 32)[0] >= 254);

    let mut opaque = PointPass::new(
        &h.context,
        &mut h.shaders,
        config(PointBlend::Opaque, false),
    );
    opaque.set_points(&h.context, &[half, half]);
    assert!(px(&draw_and_save!(&mut h, &mut opaque, "opaque"), N, 32, 32)[0].abs_diff(128) <= 1);
}



#[tokio::test]
async fn zero_points_draw_nothing_and_do_not_panic()
{
    let mut h = Harness::new().await;
    let mut pass = PointPass::new(&h.context, &mut h.shaders, config(PointBlend::Opaque, true));
    assert_eq!(lit_count(&draw(&mut h, &mut pass), N), 0);
    pass.set_points(&h.context, &[]);
    assert_eq!(lit_count(&draw(&mut h, &mut pass), N), 0);
}



#[named]
#[tokio::test]
async fn growing_past_capacity_keeps_every_point()
{
    let mut h = Harness::new().await;
    let cfg = PointPassConfig {
        capacity: 1,
        ..config(PointBlend::Opaque, true)
    };
    let mut pass = PointPass::new(&h.context, &mut h.shaders, cfg);
    pass.set_points(&h.context, &[pt(0.0, 0.0, -5.0, 1.0, [1.0; 4])]);
    assert_eq!(
        runs_in_row(&draw_and_save!(&mut h, &mut pass, "a"), N, 32),
        1
    );

    pass.set_points(
        &h.context,
        &[
            pt(-2.0, 0.0, -5.0, 1.0, [1.0; 4]),
            pt(2.0, 0.0, -5.0, 1.0, [1.0; 4]),
        ],
    );
    assert!(pass.capacity() >= 2);
    assert_eq!(pass.count(), 2);
    assert_eq!(
        runs_in_row(&draw_and_save!(&mut h, &mut pass, "b"), N, 32),
        2
    );
}



#[named]
#[tokio::test]
async fn shrinking_the_point_count_hides_the_old_points()
{
    let mut h = Harness::new().await;
    let mut pass = PointPass::new(&h.context, &mut h.shaders, config(PointBlend::Opaque, true));
    let many: Vec<Point> = (0..10)
        .map(|i| pt(i as f32 - 5.0, 0.0, -10.0, 1.0, [1.0; 4]))
        .collect();
    pass.set_points(&h.context, &many);
    pass.set_points(&h.context, &many[..1]);
    assert_eq!(runs_in_row(&draw_and_save!(&mut h, &mut pass), N, 32), 1);
}



#[tokio::test]
async fn view_color_mask_applies_to_points()
{
    let mut h = Harness::new().await;
    let mut pass = PointPass::new(&h.context, &mut h.shaders, config(PointBlend::Opaque, true));
    pass.set_points(&h.context, &[pt(0.0, 0.0, -5.0, 2.0, [1.0; 4])]);
    let views = [cam_view().mask(wgpu::ColorWrites::GREEN)];
    let out = h.render_rgba8((N, N), &views, &[&pass]);

    assert_eq!(px(&out, N, 32, 32), GREEN);
}



#[named]
#[tokio::test]
async fn points_written_by_a_compute_shader_can_be_drawn()
{
    let mut h = Harness::new().await;
    let id = h
        .shaders
        .load_shader_source(&h.context, include_str!("shaders/points_fill.wgsl"))
        .id();
    let pipeline = ComputePipelineDesc::new(id)
        .create_pipeline(&h.context, &h.shaders)
        .unwrap();
    let points = Buffer::<PointUniform>::new(
        &h.context,
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        3,
        BufferAlignment::UniformAlignment,
    );
    let bg = BindGroupBuilder::new()
        .add_entry(&points)
        .build_compute(&h.context, &pipeline);
    let mut enc = h.context.device.create_command_encoder(&Default::default());
    {
        let mut pass = enc.begin_compute_pass(&Default::default());
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bg, &[]);
        pass.dispatch_workgroups(1, 1, 1);
    }
    h.context.queue.submit([enc.finish()]);

    let mut pass = PointPass::new(&h.context, &mut h.shaders, config(PointBlend::Opaque, true));
    pass.use_buffer(points);
    assert_eq!(pass.count(), 3);
    let out = draw_and_save!(&mut h, &mut pass);
    assert_eq!(runs_in_row(&out, N, 32), 3, "x = -2, 0, +2");
}



#[named]
#[tokio::test]
async fn one_hundred_thousand_points_render()
{
    // Smoke test for use case 1: no panic, something visible, finishes quickly.
    let mut h = Harness::new().await;
    let mut pass = PointPass::new(
        &h.context,
        &mut h.shaders,
        config(PointBlend::Additive, false),
    );
    let pts: Vec<Point> = (0..100_000u32)
        .map(|i| {
            let t = i as f32 * 0.001;
            pt(
                t.sin() * 3.0,
                t.cos() * 3.0,
                -6.0 - (i % 7) as f32,
                0.05,
                [0.2, 0.4, 1.0, 1.0],
            )
        })
        .collect();
    pass.set_points(&h.context, &pts);
    assert_eq!(pass.count(), 100_000);
    let out = draw_and_save!(&mut h, &mut pass);
    assert!(lit_count(&out, N) > 100);
}
