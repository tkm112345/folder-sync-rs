use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = std::env::var("OUT_DIR")?;
    
    // copy config.json
    let dest_path = Path::new(&out_dir).join("../../../config.json");
    fs::copy("./src/config.json", dest_path)?;

    // copy log4rs.yaml
    let dest_path = Path::new(&out_dir).join("../../../log4rs.yaml");
    fs::copy("./log4rs.yaml", dest_path)?;


    // copy license
    let dest_path = Path::new(&out_dir).join("../../../LICENSE");
    fs::copy("./LICENSE", dest_path)?;

    Ok(())
}
