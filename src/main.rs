use file_server::TcpServer;

use std::io::Result;

fn main() -> Result<()> {
  let server = TcpServer::new(9000);

  server.start_server()?;
  Ok(())
}
