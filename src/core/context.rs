pub struct Context
{
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}



#[derive(Default)]
pub struct ContextBuilder
{
    pub backends: wgpu::Backends,
    pub features: wgpu::Features,
    pub limits: wgpu::Limits,
    pub power: wgpu::PowerPreference,
}



impl Context
{
    fn create_instance(b: &ContextBuilder) -> wgpu::Instance
    {
        wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: b.backends,
            flags: wgpu::InstanceFlags::default(),
            memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
            backend_options: wgpu::BackendOptions::from_env_or_default(),
            display: None,
        })
    }



    async fn new(
        b: ContextBuilder,
        instance: wgpu::Instance,
        surface: Option<&wgpu::Surface<'static>>,
    ) -> anyhow::Result<Self>
    {
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: b.power,
                compatible_surface: surface,
                force_fallback_adapter: false,
                apply_limit_buckets: true,
            })
            .await?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: b.features,
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                required_limits: b.limits,
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await?;

        Ok(Self {
            instance,
            adapter,
            device,
            queue,
        })
    }



    pub async fn headless(b: ContextBuilder) -> anyhow::Result<Self>
    {
        let instance = Self::create_instance(&b);

        Self::new(b, instance, None).await
    }



    pub async fn with_surface(
        b: ContextBuilder,
        win: impl Into<wgpu::SurfaceTarget<'static>>,
    ) -> anyhow::Result<(Self, wgpu::Surface<'static>)>
    {
        let instance = Self::create_instance(&b);

        let surface = instance.create_surface(win)?;

        Ok((Self::new(b, instance, Some(&surface)).await?, surface))
    }
}
