pub use glam::camera::rh::view as glam_view;



pub type Mat4 = [[f32; 4]; 4];
pub type Vec4 = [f32; 4];



pub const DEG_TO_RAD: f32 = std::f32::consts::PI / 180.0f32;



pub fn look_at<V: Into<glam::Vec3>>(eye: V, center: V, up: V) -> glam::Affine3
{
    glam_view::look_at_affine3(eye.into(), center.into(), up.into())
}



pub fn identity() -> Mat4
{
    [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 0.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}



pub fn mul(a: &Mat4, b: &Mat4) -> Mat4
{
    (glam::Mat4::from_cols_array_2d(a) * glam::Mat4::from_cols_array_2d(b)).to_cols_array_2d()
}
