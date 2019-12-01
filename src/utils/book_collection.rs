use super::http_tokenlize::get_id;
use book_collection_error::*;
use mysql as my;
pub mod book_collection_error {
    #[derive(Debug)]
    pub struct BookAddError {
        pub e: String,
    }
    #[derive(Debug)]
    pub struct CollectBookError {
        pub e: String,
    }
    #[derive(Debug)]
    pub struct UserAddError;

    #[derive(Debug, PartialOrd, PartialEq)]
    pub struct CannotLocateBook {
        pub e: String,
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
}
pub struct BookBuilder {
    bid: i32,
    title: String,
    author: String,
    url: String,
}

impl BookBuilder {
    pub fn new() -> Self {
        BookBuilder {
            bid: 0,
            title: "".to_string(),
            author: "".to_string(),
            url: "".to_string(),
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

pub fn add_book(book: &Book, pool: &mysql::Pool) -> Result<(), BookAddError> {
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

pub fn add_user(uid: i32, pool: &mysql::Pool) -> Result<(), UserAddError> {
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

pub fn collect(uid: i32, bid: i32, pool: &mysql::Pool) -> Result<(), CollectBookError> {
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
            return Err(CollectBookError {
                e: "The book has already collected.".to_string(),
            });
        }
    }

    for mut stmt in pool
        .prepare(
            r"INSERT INTO Collects
                        (uid, bid)
                   VALUES
                        (:uid, :bid",
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

pub fn find_book(bid: i32, pool: &mysql::Pool) -> Result<String, CannotLocateBook> {
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
        return Err(CannotLocateBook {
            e: format!("Cannot locate the book:{}.", bid),
        });
    } else {
        return Ok(urls.pop().unwrap());
    }
}
