use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    tonic_build::compile_protos("proto/grpc_worker.proto")?;
    println!("cargo:rerun-if-changed=proto/grpc_worker.proto");

    Ok(())
}
