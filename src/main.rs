#[macro_use]
extern crate mysql;
pub mod utils;
use mysql as my;

use std::fs;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};

use crate::utils::http_tokenlize::{get_method, get_response, tokenize};
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
    let pool = database_init();
    let method = get_method(&buffer[..]);
    get_response(&mut stream, method);
}
