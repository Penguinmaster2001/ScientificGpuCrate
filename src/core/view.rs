use crate::core::{
    buffer::{
        Buffer,
        BufferAlignment,
    },
    context::Context,
    math::{
        DEG_TO_RAD,
        Mat4,
        Vec4,
    },
    uniform::Uniform,
};



use glam::camera::rh::proj::directx as glam_proj;



#[derive(Clone, Copy)]
pub enum ViewProjection
{
    Matrix(glam::Mat4),
    Perspective
    {
        fov_deg: f32,
    },
    PerspectiveInfRev
    {
        fov_deg: f32,
    },
}



impl From<Mat4> for ViewProjection
{
    fn from(value: Mat4) -> Self
    {
        Self::Matrix(glam::Mat4::from_cols_array_2d(&value))
    }
}



impl From<glam::Mat4> for ViewProjection
{
    fn from(value: glam::Mat4) -> Self
    {
        Self::Matrix(value)
    }
}



impl ViewProjection
{
    pub fn proj90() -> Self
    {
        Self::Perspective { fov_deg: 90.0 }
    }



    pub fn matrix(&self, aspect_ratio: f32, near: f32, far: f32) -> glam::Mat4
    {
        match self
        {
            ViewProjection::Matrix(mat4) => *mat4,
            ViewProjection::Perspective { fov_deg } =>
            {
                glam_proj::perspective(DEG_TO_RAD * fov_deg, aspect_ratio, near, far)
            }
            ViewProjection::PerspectiveInfRev { fov_deg } =>
            {
                glam_proj::perspective_infinite_reverse(DEG_TO_RAD * fov_deg, aspect_ratio, near)
            }
        }
    }
}



pub struct View
{
    pub view: glam::Affine3,
    pub proj: ViewProjection,
    pub color_mask: wgpu::ColorWrites,
}



impl View
{
    pub fn new<A: Into<glam::Affine3>, M: Into<ViewProjection>>(view: A, proj: M) -> Self
    {
        Self {
            view: view.into(),
            proj: proj.into(),
            color_mask: wgpu::ColorWrites::all(),
        }
    }



    pub fn mask(mut self, m: wgpu::ColorWrites) -> Self
    {
        self.color_mask = m;
        self
    }
}



#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Clone, Copy, Debug)]
pub struct ViewUniform
{
    pub view: Mat4,
    pub proj: Mat4,
    pub view_proj: Mat4,
    pub inv_view: Mat4,
    pub inv_proj: Mat4,
    pub position: Vec4,
    pub viewport: Vec4,
    pub near_far: Vec4,
}



impl ViewUniform
{
    pub fn new<R: Into<(u32, u32)>>(view: &View, viewport: R) -> Self
    {
        let viewport = viewport.into();
        let width = viewport.0.max(1);
        let height = viewport.1.max(1);

        let view_mat4 = glam::Mat4::from(view.view);
        let view_inv = view.view.inverse();
        let proj = view
            .proj
            .matrix(viewport.0 as f32 / height as f32, 0.5, 1000.0);

        Self {
            view: view_mat4.to_cols_array_2d(),
            proj: proj.to_cols_array_2d(),
            view_proj: (proj * view.view).to_cols_array_2d(),
            inv_view: view_mat4.inverse_or_zero().to_cols_array_2d(),
            inv_proj: proj.inverse_or_zero().to_cols_array_2d(),
            position: view_inv.translation.extend(1.0f32).to_array(),
            viewport: [
                width as _,
                height as _,
                1.0f32 / width as f32,
                1.0f32 / height as f32,
            ],
            near_far: [0.5, 1000.0, 0.0, 0.0],
        }
    }
}



impl Uniform for ViewUniform {}



pub struct ViewBuffer
{
    buffer: Buffer<ViewUniform>,
    bind_group: wgpu::BindGroup,
    layout: wgpu::BindGroupLayout,
    stride: u64,
}



impl ViewBuffer
{
    pub fn new(context: &Context) -> Self
    {
        let buffer = Buffer::new(
            context,
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            1,
            BufferAlignment::UniformAlignment,
        );
        let stride = buffer.stride();

        let layout = context
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT | wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("view_bind_group_layout"),
            });

        let bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: buffer.handle(),
                        offset: 0,
                        size: wgpu::BufferSize::new(ViewUniform::padded_uniform_size(context)),
                    }),
                }],
                label: Some("view_bind_group"),
            });

        Self {
            buffer,
            bind_group,
            layout,
            stride,
        }
    }



    fn resize_buffer(&mut self, context: &Context, count: u64)
    {
        self.buffer = Buffer::new(
            context,
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            count,
            BufferAlignment::UniformAlignment,
        );

        self.bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: self.buffer.handle(),
                        offset: 0,
                        size: wgpu::BufferSize::new(ViewUniform::padded_uniform_size(context)),
                    }),
                }],
                label: Some("view_bind_group"),
            });
    }



    pub fn upload(&mut self, context: &Context, views: &[ViewUniform])
    {
        if views.len() > self.buffer.count() as _
        {
            self.resize_buffer(context, views.len() as _);
        }

        self.buffer.write(context, 0, views);
    }



    pub fn offset(&self, i: usize) -> u32
    {
        (i as u64 * self.stride()) as u32
    }



    pub fn bind_group(&self) -> &wgpu::BindGroup
    {
        &self.bind_group
    }



    pub fn layout(&self) -> &wgpu::BindGroupLayout
    {
        &self.layout
    }



    pub fn stride(&self) -> u64
    {
        self.stride
    }
}
