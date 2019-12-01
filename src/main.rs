#[macro_use]
extern crate mysql;
use mysql as my;

use std::fs;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};

mod utils;
use crate::utils::http_tokenlize::{get_id, get_response, tokenize};
use utils::book_collection::*;
use utils::http_tokenlize;
use utils::thread_pool::{PoolCreationError, ThreadPool};

fn main() {
    let pool = database_init();

    let listener = TcpListener::bind("127.0.0.1:7889").unwrap();
    let pool = match ThreadPool::new(4) {
        Ok(pool) => pool,
        Err(error) => panic!(error.e),
    };
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        pool.execute(|| {
            handle_connection(stream);
        });
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 512];

    stream.read(&mut buffer).unwrap();
    /*
    let get = b"GET / HTTP/1.1\r\n";
    //tokenize(&buffer);
    let (status_line, filename) = if buffer.starts_with(get) {
        ("HTTP/1.1 200 OK\r\n\r\n", "html/hello.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND\r\n\r\n", "html/404.html")
    };

    let contents = fs::read_to_string(filename).unwrap();
    let response = format!("{}{}", status_line, contents);
    */
    let pool = database_init();
    get_response(
        &mut stream,
        find_book(get_id(&buffer[..]).parse::<i32>().unwrap(), &pool).unwrap(),
    );
}
