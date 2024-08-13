use std::{
    io::prelude::*,
    net::{TcpListener, TcpStream},
};

fn response_builder(text: String) -> String {
    let body = format!(
        "<!DOCTYPE html>\
<html>\
<head>\
    <title>Hello</title>\
</head>\
<body>\
    <h1>Hello, World!</h1>\
    <p>{}</p>\
</body>\
</html>",
        text
    );

    let content_length = body.len();

    let response = format!(
        "HTTP/1.1 200 OK\r\n\
Content-Type: text/html; charset=UTF-8\r\n\
Content-Length: {}\r\n\
Date: Mon, 12 Aug 2024 10:15:30 GMT\r\n\
Server: CustomRustServer/0.1\r\n\
\r\n\
{}",
        content_length, body
    );

    response
}

fn handle_client(mut stream: TcpStream) -> std::io::Result<()> {
    let mut buf = [0; 1024];
    let mut request = String::new();

    // Read the headers
    loop {
        let bytes_read = stream.read(&mut buf)?;

        request.push_str(&String::from_utf8_lossy(&buf[..bytes_read]));

        if request.contains("\r\n\r\n") || bytes_read == 0 {
            break;
        }
    }

    // Parse the request (Simplified)
    let req: Vec<&str> = request.split_whitespace().collect();
    let method = req[0];
    let path = req[1];
    let http_version = req[2];

    let host = req[4];
    let content_type = req[6];

    if method == "POST" && content_type != "application/json" {
        // Parse the Content-Length Header
        let mut content_length = 0;

        for line in request.lines() {
            if line.starts_with("Content-Length: ") {
                content_length = match line.split(": ").nth(1) {
                    Some(value) => value.parse().unwrap_or(0),
                    None => 0,
                };

                break;
            }
        }

        let mut body = Vec::new();
        let mut bytes_read = 0;

        while bytes_read < content_length {
            let max_to_read = std::cmp::min(buf.len(), content_length - bytes_read);
            let bytes_from_stream = stream.read(&mut buf[..max_to_read])?;

            if bytes_from_stream == 0 {
                break; // EOF
            }

            body.extend_from_slice(&buf[..bytes_from_stream]);
            bytes_read += bytes_from_stream;
        }

        let text = format!(
            "You are sending a form data with this info: {}",
            String::from_utf8_lossy(&body)
        );

        let mut response_text = String::new();

        match text.split("; ").nth(1) {
            Some(value) => {
                let values: Vec<_> = value.split_whitespace().collect();

                let new_str = format!("{}, value={}", values[0], values[1]);

                response_text.push_str(new_str.as_str());
            }
            None => response_text.push_str("")
        };

        let text = format!(
            "You are sending a form data with this info: {}",
            response_text
        );
        let response = response_builder(text);

        match stream.write_all(response.as_bytes()) {
            Err(e) => {
                eprintln!("Failed to write response: {}", e);
                return Err(e);
            }
            _ => (),
        }

        match stream.flush() {
            Err(e) => {
                eprintln!("Failed to flush stream: {}", e);
                return Err(e);
            }
            _ => (),
        }

        return Ok(());
    }

    let response = response_builder(format!(
        "This is a {} request with {} path and the http version is: {}",
        method, path, http_version
    ));

    stream.write_all(response.as_bytes())?;
    let _ = stream.flush();

    Ok(())
}

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:3000")?;

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_client(stream)?;
            }
            Err(e) => {
                println!("{:#?}", e);
            }
        }
    }

    Ok(())
}
