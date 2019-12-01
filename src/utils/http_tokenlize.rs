use http::{Request, Response, StatusCode, Version};
use std::fs;
use std::io::prelude::*;
use std::net::{IpAddr, Ipv4Addr, TcpStream};

pub fn tokenize(buffer: &[u8]) -> Request<()> {
    let s = String::from_utf8_lossy(&buffer[..]);
    let rows: Vec<String> = s
        .split("\r\n")
        .filter(|s| s.len() > 0)
        .map(|s| s.to_string())
        .collect();
    let (method, uri, version): (String, String, String) = get_muv(rows[0].as_str());

    Request::builder()
        .method(method.as_str())
        .uri(uri)
        .version(http::Version::default())
        .body(())
        .unwrap()
}

fn get_muv(s: &str) -> (String, String, String) {
    let muv: Vec<&str> = s.split_whitespace().collect();
    (muv[0].to_string(), muv[1].to_string(), muv[2].to_string())
}

pub fn get_id(buffer: &[u8]) -> String {
    let request = tokenize(&buffer);
    if request.method() == http::method::Method::GET {
        if request.uri().path().starts_with("/book/id") {
            return request.uri().query().unwrap().to_string();
        } else {
            return String::from("");
        }
    } else {
        return String::from("");
    }
}

pub fn get_response(stream: &mut TcpStream, book_path: String) {
    let (status_line, contents) = match fs::read_to_string(book_path) {
        Ok(contents) => ("HTTP/1.1 200 OK\r\n\r\n", contents),
        Err(_) => ("HTTP/1.1 404 NOT FOUND\r\n\r\n", "".to_string()),
    };

    let response = format!("{}{}", status_line, contents);

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
