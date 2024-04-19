use itertools::Itertools;
use std::{collections::HashMap, io::{BufRead, Read, Write}, net::TcpListener, env, fs};
use std::thread;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        let _worker = thread::spawn(
            move || {
                match stream {
                    Ok(mut stream) => {
                        let mut buffer = [0; 1024];
                        stream.read(&mut buffer).unwrap();
        
                        let http_request: Vec<_> = buffer
                            .lines()
                            .map(|r| r.unwrap())
                            .take_while(|line| !line.is_empty())
                            .collect();
                        let (method, path, _version) = http_request
                            .first()
                            .unwrap()
                            .split(" ")
                            .take(3)
                            .collect_tuple()
                            .unwrap();
        
                        let mut headers: HashMap<&str, &str> = HashMap::new();
                        for header in http_request[1..].into_iter() {
                            let (header_name, header_value) = header.split(":").map(|s| s.trim()).take(2).collect_tuple().unwrap();
                            headers.entry(header_name).or_insert(header_value);
                        }

                        let mut read_filename: &str = "";
                        let mut write_filename: &str = "";
        
                        let (status_line, content) = match (method, path) {
                            ("GET", "/") => ("HTTP/1.1 200 OK", "OK"),
                            ("GET", a) => {
                                if a.starts_with("/echo/") {
                                    let word = a.trim_start_matches("/echo/");
                                    ("HTTP/1.1 200 OK", word)
                                } else if a.starts_with("/user-agent") {
                                    let user_agent = *headers.get("User-Agent").unwrap();
                                    ("HTTP/1.1 200 OK", user_agent)
                                } else if a.starts_with("/files/") {
                                    read_filename = a.trim_start_matches("/files/");
                                    ("HTTP/1.1 200 OK", "")
                                } else {
                                    ("HTTP/1.1 404 Not Found", "Not Found")
                                }
                            },
                            ("POST", a) => {
                                if a.starts_with("/files/") {
                                    write_filename = a.trim_start_matches("/files/");
                                    ("HTTP/1.1 201 Created", "")
                                } else {
                                    ("HTTP/1.1 404 Not Found", "Not Found")
                                }
                            },
                            _ => {
                                ("HTTP/1.1 404 Not Found", "Not Found")
                            }
                        };
                        
                        let response;
                        if read_filename.len() > 0 {
                            let args = env::args().collect_vec();
                            let directory = args.get(2).unwrap().trim_end_matches("/");
                            let read_filename = format!("{}/{}", directory, read_filename);
                            match fs::read_to_string(read_filename) {
                                Ok(file_content) => {
                                    response = format!(
                                        "{}\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n{}\r\n",
                                        status_line,
                                        file_content.len(),
                                        file_content
                                    );
                                },
                                Err(_e) => {
                                    response = format!(
                                        "{}\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}\r\n",
                                        "HTTP/1.1 404 Not Found",
                                        "".len(),
                                        ""
                                    );
                                }
                            }
                        } else if write_filename.len() > 0 {
                            let args = env::args().collect_vec();
                            let directory = args.get(2).unwrap().trim_end_matches("/");
                            let write_filename = format!("{}/{}", directory, write_filename);
                            let content = String::from_utf8(buffer.to_vec()).unwrap();
                            let content = content.split("\r\n").last().unwrap().replace("\x00", "");
                            match fs::write(write_filename, content.clone()) {
                                Ok(()) => {
                                    response = format!(
                                        "{}\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n{}\r\n",
                                        status_line,
                                        content.len(),
                                        content
                                    );
                                },
                                Err(_e) => {
                                    response = format!(
                                        "{}\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}\r\n",
                                        "HTTP/1.1 404 Not Found",
                                        "".len(),
                                        ""
                                    );
                                }
                            }
                        } else {
                            response = format!(
                                "{}\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}\r\n",
                                status_line,
                                content.len(),
                                content
                            );
                        }
                        if stream.write_all(response.as_bytes()).is_err() {
                            println!("Error writing to stream");
                        }
                    }
                    Err(e) => {
                        println!("error: {}", e);
                    }
                }
            }
        );
    }
}
