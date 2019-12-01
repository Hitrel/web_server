pub mod book_collection;
pub mod http_tokenlize;
pub mod thread_pool;

#[cfg(test)]
mod test {
    static BUFFER: &[u8] =  b"GET /book/id?1 HTTP/1.1\r\nHost: 127.0.0.1:7889\r\nConnection: keep-alive\r\nCache-Control: max-age=0\r\nUpgrade-Insecure-Requests: 1\r\nUser-Agent: Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/78.0.3904.97 Safari/537.36\r\nSec-Fetch-User: ?1\r\nAccept: text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3\r\nSec-Fetch-Site: none\r\nSec-Fetch-Mode: navigate\r\nAccept-Encoding: gzip, deflate, br\r\nAccept-Language: en,zh;q=0.9,zh-CN;q=0.";

    #[test]
    fn test() {
        use crate::utils::http_tokenlize::get_id;
        assert_eq!("1".to_string(), get_id(&BUFFER[..]));
    }

    #[test]
    fn test_add_book() {
        use crate::utils::book_collection::{add_book, database_init, find_book, BookBuilder};

        let pool = database_init();
        let book = BookBuilder::new()
            .title("The Book")
            .author("The Author")
            .url(r"/resources/1.txt")
            .build();
        if let Err(_) = find_book(1, &pool) {
            add_book(&book, &pool).unwrap();
        }
        assert_eq!(Ok(r"/resources/1.txt".to_string()), find_book(1, &pool));
    }

    #[test]
    fn test_query_get_book() {
        use crate::utils::book_collection::find_book;
        use crate::utils::http_tokenlize::get_id;
        let bid: i32 = get_id(&BUFFER[..]).parse::<i32>().unwrap();
        assert_eq!(bid, 1);
    }

    #[test]
    fn read_book() {
        use std::fs;
        use std::io::prelude::*;
        let contents = fs::read_to_string("resources/1.txt").unwrap();
        println!("{}", contents);
    }
}
