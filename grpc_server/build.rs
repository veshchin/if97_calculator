fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Указываем Cargo пересобирать gRPC-код только при изменении .proto файлов
    println!("cargo:rerun-if-changed=proto/");

    // Компиляция gRPC интерфейсов
    tonic_build::configure()
        .build_server(true)
        .compile_protos(&["proto/if97.proto"], &["proto/"])?;

    Ok(())
}