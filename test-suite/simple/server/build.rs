use std::io;

fn main() -> io::Result<()> {
    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .compile_protos(&["echo.proto"], &["../proto"])
}
