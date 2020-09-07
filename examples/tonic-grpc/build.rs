use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    tonic_build::compile_protos("proto/tonic-grpc.proto")?;
    println!("cargo:rerun-if-changed=proto/tonic-grpc.proto");

    Ok(())
}
