use std::{io::Write, io::Read, net::TcpListener};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let mut buffer = [0; 1024];
                stream.read(&mut buffer).unwrap();

                let index = b"GET / HTTP/1.1\r\n";
                let status_line = if buffer.starts_with(index) {
                    "HTTP/1.1 200 OK"
                } else {
                    "HTTP/1.1 404 Not Found"
                };
                
                let response = format!("{}\r\n\r\n", status_line);
                if stream.write_all(response.as_bytes()).is_err() {
                    println!("Error writing to stream");
                }
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
