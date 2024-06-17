fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .protoc_arg("--experimental_allow_proto3_optional")
        .build_client(false)
        .compile(
            &["proto/plugin.proto", "proto/tfplugin6.5.proto"],
            &["proto"],
        )?;
    Ok(())
}
