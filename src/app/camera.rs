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
    pub forward: glam::Vec3,
    pub up: glam::Vec3,
}



impl OrbitCamera
{
    pub fn new<
        V: Into<glam::Vec3>,
        F: Into<glam::Vec3>,
        U: Into<glam::Vec3>,
        P: Into<ViewProjection>,
    >(
        position: V,
        distance: f32,
        forward: F,
        up: U,
        projection: P,
    ) -> Self
    {
        Self {
            projection: projection.into(),
            target: position.into(),
            distance,
            forward: forward.into(),
            up: up.into(),
        }
    }
}



impl Camera for OrbitCamera
{
    fn view(&self) -> View
    {
        View::new(
            math::look_at(self.distance * -self.forward, self.target, self.up),
            self.projection,
        )
    }
}



pub struct FreeCamera
{
    pub projection: ViewProjection,
    pub transform: glam::Affine3,
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
}



impl Camera for FreeCamera
{
    fn view(&self) -> View
    {
        View::new(self.transform, self.projection)
    }
}
