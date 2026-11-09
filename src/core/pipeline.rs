use crate::core::{
    Error,
    bind::VertexLayoutDesc,
    context::Context,
    shader::{
        ShaderId,
        ShaderLib,
    },
};
use pollster::FutureExt;
use std::{
    collections::HashMap,
    sync::Arc,
};



// RenderPipelineDesc, ComputePipelineDesc, PipelineCache (Hash + Eq keys)



#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct DepthDesc
{
    format: wgpu::TextureFormat,
    depth_compare: wgpu::CompareFunction,
    depth_write_enabled: bool,
}



impl DepthDesc
{
    pub fn reversed_z() -> Self
    {
        Self {
            format: wgpu::TextureFormat::Depth32Float,
            depth_compare: wgpu::CompareFunction::Greater,
            depth_write_enabled: true,
        }
    }



    pub fn always() -> Self
    {
        Self {
            format: wgpu::TextureFormat::Depth32Float,
            depth_compare: wgpu::CompareFunction::Always,
            depth_write_enabled: true,
        }
    }
}



#[derive(Clone, PartialEq, Eq, Hash)]
pub struct RenderPipelineDesc
{
    pub shader: ShaderId,
    pub vs: &'static str,
    pub fs: Option<&'static str>,
    pub vertex: Vec<VertexLayoutDesc>, // owned form of VertexBufferLayout
    pub topology: wgpu::PrimitiveTopology,
    pub cull: Option<wgpu::Face>,
    pub front_face: wgpu::FrontFace,
    pub depth: Option<DepthDesc>,             // format, compare, write
    pub targets: Vec<wgpu::ColorTargetState>, // format, blend, write mask
    pub samples: u32,
    pub layouts: Vec<wgpu::BindGroupLayout>,
}



impl RenderPipelineDesc
{
    pub fn new(shader: ShaderId, vs: &'static str, fs: Option<&'static str>) -> Self
    {
        Self {
            shader,
            vs,
            fs,
            vertex: vec![],
            topology: wgpu::PrimitiveTopology::TriangleList,
            cull: Some(wgpu::Face::Back),
            front_face: wgpu::FrontFace::Ccw,
            depth: None,
            targets: vec![],
            samples: 1,
            layouts: vec![],
        }
    }



    pub fn color_target(
        mut self,
        format: wgpu::TextureFormat,
        blend: Option<wgpu::BlendState>,
    ) -> Self
    {
        self.targets.push(wgpu::ColorTargetState {
            format,
            blend,
            write_mask: wgpu::ColorWrites::default(),
        });

        self
    }



    pub fn depth(mut self, depth_desc: DepthDesc) -> Self
    {
        self.depth = Some(depth_desc);

        self
    }



    pub fn front_face(mut self, front_face: wgpu::FrontFace) -> Self
    {
        self.front_face = front_face;

        self
    }



    pub fn layout(mut self, layout: wgpu::BindGroupLayout) -> Self
    {
        self.layouts.push(layout);

        self
    }



    pub fn color_mask(mut self, color: wgpu::ColorWrites) -> Self
    {
        if let Some(target) = self.targets.last_mut()
        {
            target.write_mask = color;
        }

        self
    }
}



pub struct ComputePipelineDesc
{
    pub shader: ShaderId,
    // pub layouts: Vec<LayoutId>,
}



impl ComputePipelineDesc
{
    pub fn new(shader: ShaderId) -> Self
    {
        Self { shader }
    }

    pub fn create_pipeline(
        &self,
        context: &Context,
        shader_lib: &ShaderLib,
    ) -> anyhow::Result<wgpu::ComputePipeline>
    {
        Ok(context
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: None,
                layout: None,
                module: shader_lib
                    .get_shader(self.shader)
                    .ok_or(Error::NotFound)?
                    .module(),
                entry_point: None,
                compilation_options: Default::default(),
                cache: Default::default(),
            }))
    }
}



pub struct PipelineCache
{
    cache: HashMap<RenderPipelineDesc, Arc<wgpu::RenderPipeline>>,
}



impl PipelineCache
{
    pub fn new() -> Self
    {
        Self {
            cache: HashMap::new(),
        }
    }



    pub fn render(
        &mut self,
        context: &Context,
        shaders: &ShaderLib,
        desc: &RenderPipelineDesc,
    ) -> anyhow::Result<Arc<wgpu::RenderPipeline>>
    {
        if let Some(pipeline) = self.cache.get(desc)
        {
            return Ok(pipeline.clone());
        }

        let depth_stencil = desc.depth.map(|d| wgpu::DepthStencilState {
            format: d.format,
            depth_write_enabled: Some(d.depth_write_enabled),
            depth_compare: Some(d.depth_compare),
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        });

        let shader = shaders.get_shader(desc.shader).ok_or(Error::NotFound)?;

        let fragment = if !desc.targets.is_empty() || desc.fs.is_some()
        {
            Some(wgpu::FragmentState {
                module: shader.module(),
                entry_point: desc.fs,
                targets: &desc.targets.iter().cloned().map(Some).collect::<Vec<_>>(),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            })
        }
        else
        {
            None
        };

        let error_scope = context
            .device
            .push_error_scope(wgpu::ErrorFilter::Validation);

        let pipeline = Arc::new(
            context
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: None,
                    layout: Some(&context.device.create_pipeline_layout(
                        &wgpu::PipelineLayoutDescriptor {
                            label: None,
                            bind_group_layouts: &desc.layouts.iter().map(Some).collect::<Vec<_>>(),
                            immediate_size: 0,
                        },
                    )),

                    vertex: wgpu::VertexState {
                        module: shader.module(),
                        entry_point: Some(desc.vs),
                        buffers: &desc
                            .vertex
                            .iter()
                            .map(|v| Some(v.to_vertex_layout()))
                            .collect::<Vec<_>>(),
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                    },

                    fragment,

                    primitive: wgpu::PrimitiveState {
                        topology: desc.topology,
                        strip_index_format: None,
                        front_face: desc.front_face,
                        cull_mode: desc.cull,
                        ..Default::default()
                    },

                    depth_stencil,

                    multisample: wgpu::MultisampleState {
                        count: desc.samples,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },

                    multiview_mask: None,
                    cache: None,
                }),
        );

        error_scope.pop().block_on().map_or(Ok(()), Err)?;

        self.cache.insert(desc.clone(), pipeline.clone());

        Ok(pipeline)
    }



    pub fn len(&self) -> usize
    {
        self.cache.len()
    }



    pub fn is_empty(&self) -> bool
    {
        self.cache.is_empty()
    }
}



impl Default for PipelineCache
{
    fn default() -> Self
    {
        Self::new()
    }
}
