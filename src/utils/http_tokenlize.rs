use crate::utils::book_collection::{database_init, find_book, get_booklist, parse_book};
use encoding::all::UTF_8;
use encoding::{DecoderTrap, Encoding};
use http::{Request, Response, StatusCode, Version};
use std::fs;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::net::{IpAddr, Ipv4Addr, TcpStream};

#[derive(Debug, PartialOrd, PartialEq)]
pub enum GetMethod {
    Book(String),
    List,
    None,
}

impl GetMethod {
    pub fn unwrap(self) -> String {
        match self {
            GetMethod::Book(str) => str,
            _ => panic!("Called `unwrap`)1 on a `GetMethod::None` or `GetMethod::List` value"),
        }
    }
}

pub fn tokenize(buffer: &[u8]) -> Result<Request<()>, ()> {
    let s = String::from_utf8_lossy(&buffer[..]);
    let rows: Vec<String> = s
        .split("\r\n")
        .filter(|s| s.len() > 0)
        .map(|s| s.to_string())
        .collect();
    let (method, uri, version): (String, String, String) = match get_muv(rows[0].as_str()) {
        Some(muv) => muv,
        None => Err(()),
    };
    Ok(Request::builder()
        .method(method.as_str())
        .uri(uri)
        .version(http::Version::default())
        .body(())
        .unwrap())
}

fn get_muv(s: &str) -> Option<(String, String, String)> {
    let muv: Vec<&str> = s.split_whitespace().collect();
    if muv.len() < 2 {
        println!("{:?}", muv);
        return None;
    }
    Some((muv[0].to_string(), muv[1].to_string(), muv[2].to_string()))
}

pub fn get_method(buffer: &[u8]) -> GetMethod {
    let request = match tokenize(&buffer) {
        Ok(request) => request,
        Err(_) => return GetMethod::None,
    };
    if request.method() == http::method::Method::GET {
        if request.uri().path().starts_with("/book/id") {
            return GetMethod::Book(request.uri().query().unwrap().to_string());
        } else if request.uri().path().starts_with("/booklist") {
            return GetMethod::List;
        } else {
            return GetMethod::None;
        }
    } else {
        return GetMethod::None;
    }
}

fn file_read(path: &str) -> io::Result<String> {
    let mut f = File::open(path)?;

    let mut buffer: Vec<u8> = Vec::new();
    f.read_to_end(&mut buffer)
        .ok()
        .expect(format!("Cannot read file: {}", path).as_str());
    let content: String = UTF_8.decode(&buffer, DecoderTrap::Strict).unwrap();
    Ok(content)
}

pub fn get_response(stream: &mut TcpStream, _type: GetMethod) {
    let (status_line, contents) = match _type {
        GetMethod::Book(book_parse) => {
            let book_path = parse_book(book_parse);
            match fs::read_to_string(book_path) {
                Ok(contents) => ("HTTP/1.1 200 OK\r\n\r\n", contents),
                Err(_) => (
                    "HTTP/1.1 404 NOT FOUND\r\n\r\n",
                    "Get Book Error李".to_string(),
                ),
            }
        }
        GetMethod::List => {
            let pool = database_init();
            match get_booklist(&pool) {
                Ok(str) => ("HTTP/1.1 200 OK\r\n\r\n", str),
                Err(_) => (
                    "HTTP/1.1 404 NOT FOUND\r\n\r\n",
                    "Get Book List Error".to_string(),
                ),
            }
        }
        GetMethod::None => ("HTTP/1.1 404 NOT FOUND\r\n\r\n", "Get Nothing".to_string()),
    };

    let response = format!("{}{}", status_line, contents);

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
