use std::io;

fn main() -> io::Result<()> {
    tonic_prost_build::configure()
        .build_server(false)
        .build_transport(false)
        .build_client(true)
        .compile_protos(&["echo.proto"], &["../proto"])
}
