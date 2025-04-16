
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::fs;

fn main() {
    let listener = TcpListener::bind("0.0.0.0:8080").expect("Failed to start server");
    println!("🚀 Serving at http://localhost:8080");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let file_name = "pie_chart.html".to_string();
                handle_connection(stream, file_name);
            }
            Err(e) => eprintln!("❌ Connection failed: {}", e),
        }
    }
}

fn handle_connection(mut stream: TcpStream, file_name: String) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    let get = b"GET / HTTP/1.1\r\n";

    if buffer.starts_with(get) {
        let html_content = fs::read_to_string(file_name).expect("Failed to read file");
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: text/html\r\n\r\n{}",
            html_content.len(),
            html_content
        );
        stream.write_all(response.as_bytes()).unwrap();
    }
}
