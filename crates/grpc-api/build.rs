// Build script for gRPC code generation

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .out_dir("src/generated")
        .compile(&["proto/claw.proto"], &["."])?;
    
    println!("cargo:rerun-if-changed=proto/claw.proto");
    Ok(())
}
