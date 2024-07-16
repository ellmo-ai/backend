fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("src/code_execution.proto")?;
    Ok(())
}
