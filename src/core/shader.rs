use crate::core::context::Context;
use std::{
    collections::{
        HashMap,
        hash_map::Entry,
    },
    eprintln,
    hash::{
        DefaultHasher,
        Hash,
        Hasher,
    },
    path::{
        Path,
        PathBuf,
    },
};



#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct ShaderId
{
    id: u64,
}



impl ShaderId
{
    pub fn new(id: u64) -> Self
    {
        Self { id }
    }
}



// Source (file | embedded), concat includes, hot reload, error scopes



#[expect(unused)]
enum ShaderSource
{
    Loaded(Shader),
    File(PathBuf),
    Embedded(&'static str),
}



// TODO: Includes
#[expect(unused)]
pub struct ShaderLib
{
    includes: HashMap<String, ShaderSource>,
    map: HashMap<ShaderId, Shader>,
}



impl ShaderLib
{
    pub fn new() -> Self
    {
        Self {
            includes: HashMap::new(),
            map: HashMap::new(),
        }
    }


    pub fn get_shader(&self, id: ShaderId) -> Option<&Shader>
    {
        self.map.get(&id)
    }



    pub async fn try_load_shader_file(
        &mut self,
        context: &Context,
        path: &Path,
    ) -> anyhow::Result<&Shader>
    {
        self.try_load_shader_source(context, &std::fs::read_to_string(path)?)
            .await
    }



    pub fn load_shader_file(&mut self, context: &Context, path: &Path) -> anyhow::Result<&Shader>
    {
        Ok(self.load_shader_source(context, &std::fs::read_to_string(path)?))
    }



    pub async fn try_load_shader_source(
        &mut self,
        context: &Context,
        source: &str,
    ) -> anyhow::Result<&Shader>
    {
        // TODO: shader includes
        Ok(self.add_shader(Shader::try_from_source(context, source).await?))
    }



    pub fn load_shader_source(&mut self, context: &Context, source: &str) -> &Shader
    {
        // TODO: shader includes
        self.add_shader(Shader::from_source(context, source))
    }



    pub fn add_shader(&mut self, shader: Shader) -> &Shader
    {
        match self.map.entry(shader.id)
        {
            Entry::Vacant(entry) => entry.insert(shader),
            Entry::Occupied(mut entry) =>
            {
                *entry.get_mut() = shader;
                entry.into_mut()
            }
        }
    }



    pub fn len(&self) -> usize
    {
        self.map.len()
    }



    pub fn is_empty(&self) -> bool
    {
        self.map.is_empty()
    }



    // pub fn create_shaders<const N: usize>(context: &Context, files: &[&str; N]) -> [Shader; N]
    // {
    //     let mut n = -1;
    //     files.map(|f| {
    //         n += 1;
    //         Self::new(
    //             context
    //                 .device
    //                 .create_shader_module(wgpu::ShaderModuleDescriptor {
    //                     label: Some(format!("shader_{}", n).as_str()),
    //                     source: wgpu::ShaderSource::Wgsl(f.into()),
    //                 }),
    //         )
    //     })
    // }
}



impl Default for ShaderLib
{
    fn default() -> Self
    {
        Self::new()
    }
}



pub struct Shader
{
    id: ShaderId,
    shader_module: wgpu::ShaderModule,
}



impl Shader
{
    pub fn from_source(context: &Context, source: &str) -> Self
    {
        let shader_module = context
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(source.into()),
            });

        let mut hasher = DefaultHasher::new();
        source.hash(&mut hasher);
        let id = ShaderId::new(hasher.finish());

        Self { id, shader_module }
    }



    pub async fn try_from_source(context: &Context, source: &str) -> anyhow::Result<Self>
    {
        let error_scope = context
            .device
            .push_error_scope(wgpu::ErrorFilter::Validation);

        let shader_module = context
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(source.into()),
            });

        for message in shader_module.get_compilation_info().await.messages
        {
            eprintln!(
                "{:?} at {:?}: {}",
                message.message_type, message.location, message.message
            );
        }
        error_scope.pop().await.map_or(Ok(()), Err)?;


        let mut hasher = DefaultHasher::new();
        source.hash(&mut hasher);
        let id = ShaderId::new(hasher.finish());

        Ok(Self { id, shader_module })
    }



    pub fn module(&self) -> &wgpu::ShaderModule
    {
        &self.shader_module
    }



    pub fn id(&self) -> ShaderId
    {
        self.id
    }
}
