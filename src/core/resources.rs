use std::path::Path;



pub fn load_string(path: &Path) -> anyhow::Result<String>
{
    let txt = std::fs::read_to_string(path)?;

    Ok(txt)
}



pub fn load_binary(path: &Path) -> anyhow::Result<Vec<u8>>
{
    let data = std::fs::read(path)?;

    Ok(data)
}
