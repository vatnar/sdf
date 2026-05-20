use std::io::Result;
use sdf_server::TcpServer;

fn main() -> Result<()> {
    let server = TcpServer::new(9000);

    server.start_server()?;
    sdf_protocol::proto_add(1, 2);
    Ok(())
}
