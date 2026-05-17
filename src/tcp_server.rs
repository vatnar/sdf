use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

pub struct TcpServer{
    port: u16,
}

impl TcpServer {
    pub fn new(port: u16) -> Self {
        Self { port }
    }
    pub fn start_server(&self) -> std::io::Result<()>{
        let listener = TcpListener::bind(format!("127.0.0.1:{}", self.port))?;

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    thread::spawn(move || {
                        if let Err(e) = handle_client(stream) {
                            eprintln!("client error: {e}");
                        }
                    });
                }
                Err(e) => eprintln!("accept error: {e}"),
            }
        }
        Ok(())
    }

}

// "entry point" of application
fn handle_client(mut stream: TcpStream) -> std::io::Result<()>{
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut line = String::new();

    loop{
        line.clear();
        let n = reader.read_line(&mut line)?;
        if n == 0 {
            println!("client disconnected");
            break;
        }

        let msg = line.trim_end();
        println!("got: {:?}", msg);
        stream.write_all(b"OK\n")?;
    }
    stream.shutdown(std::net::Shutdown::Both)?;
    Ok(())
}

