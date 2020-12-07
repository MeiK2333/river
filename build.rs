fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("proto/river.proto")?;
    // cc::Build::new().file("src/memory.c").compile("memory");
    Ok(())
}
