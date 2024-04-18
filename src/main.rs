use itertools::Itertools;
use std::{io::{BufRead, Read, Write}, net::TcpListener};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let mut buffer = [0; 1024];
                stream.read(&mut buffer).unwrap();

                let http_request: Vec<_> = buffer
                    .lines()
                    .map(|r| r.unwrap())
                    .take_while(|line| !line.is_empty())
                    .collect();
                let (_method, path, _version) = http_request
                    .first()
                    .unwrap()
                    .split(" ")
                    .take(3)
                    .collect_tuple()
                    .unwrap();

                let (status_line, content) = match path {
                    "/" => ("HTTP/1.1 200 OK", "OK"),
                    a => {
                        if a.starts_with("/echo/") {
                            let word = a.trim_start_matches("/echo/");
                            ("HTTP/1.1 200 OK", word)
                        } else {
                            ("HTTP/1.1 404 Not Found", "Not Found")
                        }
                    },
                };

                let response = format!(
                    "{}\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}\r\n",
                    status_line,
                    content.len(),
                    content
                );
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
