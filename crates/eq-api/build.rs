use std::error::Error;
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn Error>> {
    let web_dist = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../web/dist");
    fs::create_dir_all(&web_dist)?;

    let index = web_dist.join("index.html");
    if !index.exists() {
        fs::write(
            index,
            "<!doctype html><html><body><p>Web UI not built. Run pnpm -C web build</p></body></html>",
        )?;
    }

    println!("cargo:rerun-if-changed=../../web/dist");
    Ok(())
}
