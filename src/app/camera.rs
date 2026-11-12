use crate::core::{
    math,
    view::{
        View,
        ViewProjection,
    },
};



pub trait Camera
{
    fn view(&self) -> View;
}



pub struct OrbitCamera
{
    pub projection: ViewProjection,
    pub target: glam::Vec3,
    pub distance: f32,
    pub to_eye: glam::Vec3,
    pub up: glam::Vec3,
}



impl Default for OrbitCamera
{
    fn default() -> Self
    {
        Self {
            projection: Default::default(),
            target: Default::default(),
            distance: 1.0,
            to_eye: glam::Vec3::Z,
            up: glam::Vec3::Y,
        }
    }
}



impl OrbitCamera
{
    pub fn new<
        V: Into<glam::Vec3>,
        F: Into<glam::Vec3>,
        U: Into<glam::Vec3>,
        P: Into<ViewProjection>,
    >(
        target: V,
        distance: f32,
        to_eye: F,
        up: U,
        projection: P,
    ) -> Self
    {
        Self {
            projection: projection.into(),
            target: target.into(),
            distance,
            to_eye: to_eye.into(),
            up: up.into(),
        }
    }



    pub fn position(&self) -> glam::Vec3
    {
        self.target + (self.distance * self.to_eye)
    }



    pub fn target(&self) -> glam::Vec3
    {
        self.target
    }



    pub fn distance(&self) -> f32
    {
        self.distance
    }



    pub fn forward(&self) -> glam::Vec3
    {
        -self.to_eye
    }



    pub fn up(&self) -> glam::Vec3
    {
        self.up
    }
}



impl Camera for OrbitCamera
{
    fn view(&self) -> View
    {
        View::new(
            math::look_at(self.position(), self.target, self.up),
            self.projection,
        )
    }
}



pub struct FreeCamera
{
    pub projection: ViewProjection,
    pub transform: glam::Affine3,
}



impl Default for FreeCamera
{
    fn default() -> Self
    {
        Self {
            projection: Default::default(),
            transform: glam::Affine3::from_cols(
                glam::Vec3::X,
                glam::Vec3::Y,
                glam::Vec3::NEG_Z,
                glam::Vec3::ZERO,
            ),
        }
    }
}



impl FreeCamera
{
    pub fn new<T: Into<glam::Affine3>, P: Into<ViewProjection>>(transform: T, projection: P)
    -> Self
    {
        Self {
            projection: projection.into(),
            transform: transform.into(),
        }
    }



    pub fn from_position_attitude<X: Into<glam::Vec3>, P: Into<ViewProjection>>(
        position: X,
        pitch: f32,
        yaw: f32,
        roll: f32,
        projection: P,
    ) -> Self
    {
        let mut c = Self::new(
            glam::Affine3::from_cols(
                glam::Vec3::X,
                glam::Vec3::Y,
                glam::Vec3::NEG_Z,
                position.into(),
            ),
            projection,
        );
        c.rotate_euler(pitch, yaw, roll);

        c
    }



    pub fn forward(&self) -> &glam::Vec3
    {
        &self.transform.z_axis
    }



    pub fn position(&self) -> &glam::Vec3
    {
        &self.transform.translation
    }



    pub fn translate_world<X: Into<glam::Vec3>>(&mut self, translation: X)
    {
        self.transform.translation += translation.into();
    }



    pub fn translate_local<X: Into<glam::Vec3>>(&mut self, translation: X)
    {
        self.transform.translation += self.transform.matrix3 * translation.into();
    }



    pub fn rotate_euler(&mut self, d_pitch: f32, d_yaw: f32, d_roll: f32)
    {
        self.transform.matrix3 =
            glam::Mat3::from_euler(glam::EulerRot::YXZEx, d_yaw, d_pitch, -d_roll)
                * self.transform.matrix3;
    }
}



impl Camera for FreeCamera
{
    fn view(&self) -> View
    {
        View::new(self.transform, self.projection)
    }
}
