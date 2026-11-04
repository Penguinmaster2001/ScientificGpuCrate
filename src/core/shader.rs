use crate::core::context::Context;
use std::{
    collections::{
        HashMap,
        hash_map::Entry,
    },
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



#[derive(Hash, PartialEq, Eq, Clone, Copy)]
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
    map: HashMap<ShaderId, ShaderSource>,
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
        match self.map.get(&id)
        {
            Some(ShaderSource::Loaded(shader)) => Some(shader),
            _ => None,
        }
    }



    pub fn load_shader_file(&mut self, context: &Context, path: &Path) -> anyhow::Result<&Shader>
    {
        Ok(self.load_shader_source(context, &std::fs::read_to_string(path)?))
    }



    pub fn load_shader_source(&mut self, context: &Context, source: &str) -> &Shader
    {
        // TODO: shader includes
        self.add_shader_module(Shader::from_source(context, source))
    }



    fn add_shader_module(&mut self, shader_module: wgpu::ShaderModule) -> &Shader
    {
        let mut hasher = DefaultHasher::new();
        shader_module.hash(&mut hasher);
        let id = ShaderId::new(hasher.finish());

        let shader = Shader { id, shader_module };
        let shader_source = ShaderSource::Loaded(shader);

        let source = match self.map.entry(id)
        {
            Entry::Vacant(entry) => entry.insert(shader_source),
            Entry::Occupied(mut entry) =>
            {
                *entry.get_mut() = shader_source;
                entry.into_mut()
            }
        };

        match source
        {
            ShaderSource::Loaded(shader) => shader,
            _ => unreachable!("inserted a Loaded shader"),
        }
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
    pub fn get_module(&self) -> &wgpu::ShaderModule
    {
        &self.shader_module
    }



    fn from_source(context: &Context, source: &str) -> wgpu::ShaderModule
    {
        context
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(source.into()),
            })
    }



    pub fn id(&self) -> ShaderId
    {
        self.id
    }
}
