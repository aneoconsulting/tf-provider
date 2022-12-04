fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("proto/plugin.proto")?;
    tonic_build::compile_protos("proto/tfplugin6.3.proto")?;
    Ok(())
}
