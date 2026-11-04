pub mod bind;
pub mod buffer;
pub mod context;
pub mod pass;
pub mod pipeline;
pub mod readback;
pub mod resources;
pub mod shader;
pub mod target;
pub mod uniform;
pub mod vertex;
pub mod view;

#[cfg(feature = "assets")]
pub mod texture;



#[derive(Debug)]
enum Error
{
    NotFound,
}



impl std::fmt::Display for Error
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        write!(f, "{:?}", self)
    }
}



impl std::error::Error for Error {}
