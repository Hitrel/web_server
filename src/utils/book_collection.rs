use super::http_tokenlize::get_method;
use json;
use mysql as my;
use std::fmt;

#[derive(Debug, PartialOrd, PartialEq)]
pub enum BookCollectionError {
    BookAddError(String),
    CollectBookError(String),
    UserAddError(String),
    BookLocateError(String),
    BookListLoadError(String),
}

type Result<T> = std::result::Result<T, BookCollectionError>;

struct JsonList(Vec<json::JsonValue>);

impl fmt::Display for JsonList {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{{}}}", self.get());
        Ok(())
    }
}

impl JsonList {
    pub fn get(&self) -> String {
        let mut s = String::new();

        for v in &self.0 {
            s = format!("{}, {}", s, v);
        }

        s.remove(0);
        s.remove(0);

        s
    }
}

pub struct User {
    uid: i32,
}

pub struct Collect {
    uid: i32,
    bid: i32,
}

pub struct Book {
    bid: i32,
    title: String,
    author: String,
    url: String,
    image_url: String,
}

impl Book {
    pub fn to_json(&self) -> json::JsonValue {
        let object = json::object! {
            "title" => self.title.clone(),
            "author" => self.author.clone(),
            "bid" => self.bid,
            "image_url" => self.image_url.clone()
        };

        object
    }
}

pub struct BookBuilder {
    bid: i32,
    title: String,
    author: String,
    url: String,
    image_url: String,
}

impl BookBuilder {
    pub fn new() -> Self {
        BookBuilder {
            bid: 0,
            title: "".to_string(),
            author: "".to_string(),
            url: "".to_string(),
            image_url: "http://a1.att.hudong.com/64/42/01300000602809125714424690146.jpg"
                .to_string(),
        }
    }

    pub fn bid(mut self, bid: i32) -> Self {
        self.bid = bid;
        self
    }

    pub fn author(mut self, author: &str) -> Self {
        self.author = author.to_string();
        self
    }

    pub fn title(mut self, title: &str) -> Self {
        self.title = title.to_string();
        self
    }

    pub fn url(mut self, url: &str) -> Self {
        self.url = url.to_string();
        self
    }

    pub fn build(&self) -> Book {
        Book {
            bid: self.bid,
            title: self.title.clone(),
            author: self.author.clone(),
            url: self.url.clone(),
            image_url: self.image_url.clone(),
        }
    }
}

pub fn database_init() -> my::Pool {
    let _my = my::Pool::new("mysql://root:root@localhost:3306/NOVELSERVER").unwrap();

    _my.prep_exec(
        r"CREATE TABLE IF NOT EXISTS Users(
                        uid INT NOT NULL PRIMARY KEY
                    )",
        (),
    )
    .unwrap();

    _my.prep_exec(
        r"CREATE TABLE IF NOT EXISTS Books(
                        bid INT AUTO_INCREMENT PRIMARY KEY,
                        title CHAR(20) NOT NULL DEFAULT '无名书',
                        author CHAR(20) NOT NULL DEFAULT '佚名',
                        url VARCHAR(100) NOT NULL
                    )",
        (),
    )
    .unwrap();
    _my.prep_exec(
        r"CREATE TABLE IF NOT EXISTS Collects(
                        uid INT,
                        bid INT,
                        PRIMARY KEY (uid, bid),
                        FOREIGN KEY (uid) REFERENCES Users(uid),
                        FOREIGN KEY (bid) REFERENCES Books(bid)
					)",
        (),
    )
    .unwrap();

    _my
}

pub fn add_book(book: &Book, pool: &mysql::Pool) -> Result<()> {
    for mut stmt in pool
        .prepare(
            r"INSERT INTO Books
                        				(title, author, url)
                   					VALUES
                        				(:title, :author, :url);",
        )
        .into_iter()
    {
        stmt.execute(params! {
            "title" => &book.title,
            "author" => &book.author,
            "url" => &book.url,
        })
        .unwrap();
    }
    Ok(())
}

pub fn add_user(uid: i32, pool: &mysql::Pool) -> Result<()> {
    for mut stmt in pool
        .prepare(
            r"INSERT INTO Users
                        		   		(uid)
                   				   VALUES
                        				(:uid",
        )
        .into_iter()
    {
        stmt.execute(params! {
            "uid" => uid,
        })
        .unwrap();
    }
    Ok(())
}

pub fn collect(uid: i32, bid: i32, pool: &mysql::Pool) -> Result<()> {
    let records: Vec<Collect> = pool
        .prep_exec(
            "SELECT uid, bid FROM Collects WHERE uid = :uid",
            params! {
            "uid" => uid,
            },
        )
        .map(|res| {
            res.map(|x| x.unwrap())
                .map(|row| {
                    let (uid, bid) = my::from_row(row);
                    Collect { uid, bid }
                })
                .collect()
        })
        .unwrap();

    for row in records {
        if row.bid == bid {
            return Err(BookCollectionError::CollectBookError(
                "The book has already collected.".to_string(),
            ));
        }
    }

    for mut stmt in pool
        .prepare(
            r"INSERT INTO Collects
                        (uid, bid)
                   VALUES
                        (:uid, :bid)",
        )
        .into_iter()
    {
        stmt.execute(params! {
            "uid" => uid,
            "bid" => bid,
        })
        .unwrap();
    }
    Ok(())
}

pub fn find_book(bid: i32, pool: &mysql::Pool) -> Result<String> {
    let mut urls: Vec<String> = pool
        .prep_exec(
            "SELECT url FROM Books WHERE bid = :bid",
            params! {
                "bid" => bid,
            },
        )
        .map(|res| {
            res.map(|x| x.unwrap())
                .map(|row| {
                    let url = my::from_row(row);
                    url
                })
                .collect()
        })
        .unwrap();

    if urls.len() != 1 {
        return Err(BookCollectionError::BookLocateError(format!(
            "Cannot locate the book:{}.",
            bid
        )));
    } else {
        return Ok(urls.pop().unwrap());
    }
}

pub fn parse_book(stem: String) -> String {
    let path = stem
        .split('=')
        .filter(|c| c.len() > 0)
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
    assert!(path.len() == 2);
    let pool = database_init();
    let bid = path[0].parse::<i32>().unwrap();
    let mut book_path = find_book(bid, &pool).unwrap();
    book_path += &path[1];

    book_path
}

pub fn get_booklist(pool: &mysql::Pool) -> Result<String> {
    let result = pool
        .prep_exec("SELECT bid, title, author FROM Books", ())
        .map(|res| {
            res.map(|x| x.unwrap())
                .map(|row| {
                    let (bid, title, author): (i32, String, String) = my::from_row(row);
                    BookBuilder::new()
                        .bid(bid)
                        .title(title.as_str())
                        .author(author.as_str())
                        .build()
                        .to_json()
                })
                .collect()
        })
        .unwrap();

    let result = JsonList(result);

    match result.0.is_empty() {
        false => Ok(format!("{}", result)),
        true => Err(BookCollectionError::BookListLoadError(
            "Cannot find book list".to_string(),
        )),
    }
}
