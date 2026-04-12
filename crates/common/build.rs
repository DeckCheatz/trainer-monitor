fn main() -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        std::env::set_var("PROTOC", protoc_bin_vendored::protoc_bin_path()?);
    }
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile(&["proto/protocol.proto"], &["proto"])?;
    Ok(())
}
