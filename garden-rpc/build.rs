use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    tonic_build::compile_protos("proto/garden_pi.proto")?;
    println!("cargo:rerun-if-changed=proto/garden_pi.proto");

    Ok(())
}
