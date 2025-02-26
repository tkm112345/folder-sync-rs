use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = std::env::var("OUT_DIR")?;
    println!("{}",out_dir);
    let dest_path = Path::new(&out_dir).join("../../../config.json");
    fs::copy("./src/config.json", dest_path)?;
    Ok(())
}