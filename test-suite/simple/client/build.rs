use std::io;

fn main() -> io::Result<()> {
    tonic_build::configure()
        .build_server(false)
        .build_client(true)
        .compile(&["echo.proto"], &["../proto"])
}
