use std::net::{SocketAddr, TcpStream};

pub struct UnconnectedTcpClient{
    addr: SocketAddr
}

pub struct ConnectedTcpClient{
   pub stream: TcpStream,
    peer: SocketAddr
}


impl UnconnectedTcpClient {
    pub fn new(addr: SocketAddr) -> Self {
        Self { addr }
    }

    pub fn connect(self) -> std::io::Result<ConnectedTcpClient>{
        let stream = TcpStream::connect(self.addr)?;
        Ok(ConnectedTcpClient{stream, peer: self.addr })
    }
}

impl ConnectedTcpClient {
    pub fn new(stream: TcpStream, peer: SocketAddr) -> Self {
        Self { stream , peer}
    }

    pub fn disconnect(self) -> std::io::Result<UnconnectedTcpClient>{
        self.stream.shutdown(std::net::Shutdown::Both)?;
        Ok(UnconnectedTcpClient{addr: self.peer})
    }
}