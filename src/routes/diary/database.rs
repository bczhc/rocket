use crate::routes::diary::{generate_id, timestamp};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::Path;

pub(crate) struct Database {
    conn: Connection,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DatabaseInfo {
    pub hash_salt: String,
}

impl Database {
    pub fn new<P: AsRef<Path>>(path: P) -> rusqlite::Result<Self> {
        let conn = Connection::open(path)?;

        conn.execute_batch(
            r#"CREATE TABLE IF NOT EXISTS user
(
    user_id     INTEGER NOT NULL PRIMARY KEY,
    username    TEXT    NOT NULL UNIQUE,
    pw_hash     TEXT    NOT NULL,
    signup_time INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS info
(
    json TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS diary
(
    id            INTEGER NOT NULL PRIMARY KEY,
    content       TEXT    NOT NULL,
    creation_time INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS diary_book
(
    id            INTEGER NOT NULL PRIMARY KEY,
    name          TEXT    NOT NULL,
    creation_time INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS diary_book_entry
(
    book_id  INTEGER NOT NULL,
    diary_id INTEGER NOT NULL,
    "date"   INTEGER NOT NULL,
    title    TEXT,
    FOREIGN KEY (book_id) REFERENCES diary_book (id),
    FOREIGN KEY (diary_id) REFERENCES diary (id)
);

CREATE TABLE IF NOT EXISTS user_diary_book
(
    user_id       INTEGER NOT NULL,
    diary_book_id INTEGER NOT NULL,
    FOREIGN KEY (diary_book_id) REFERENCES diary_book (id),
    FOREIGN KEY (user_id) REFERENCES user (user_id)
);
"#,
        )?;

        conn.execute("INSERT INTO info VALUES ('')", [])?;

        Ok(Self { conn })
    }

    pub fn check_existence(&self, username: &str, pw_hash: Option<&str>) -> bool {
        let count: u32 = match pw_hash {
            None => self
                .conn
                .query_row(
                    "SELECT COUNT() FROM user WHERE username IS ?",
                    params![username],
                    |r| r.get(0),
                )
                .unwrap(),
            Some(p) => self
                .conn
                .query_row(
                    "SELECT COUNT() FROM user WHERE username IS ? AND pw_hash IS ?",
                    params![username, p],
                    |r| r.get(0),
                )
                .unwrap(),
        };
        count != 0
    }

    pub fn fetch_info(&self) -> Option<DatabaseInfo> {
        let result: Result<String, _> =
            self.conn
                .query_row(r#"SELECT "json" FROM info"#, [], |r| r.get(0));
        match result {
            Ok(s) => serde_json::from_str(&s).ok(),
            Err(rusqlite::Error::QueryReturnedNoRows) => None,
            Err(e) => {
                panic!("{}", e)
            }
        }
    }

    // noinspection SqlWithoutWhere
    pub fn update_info(&self, info: &DatabaseInfo) {
        let json = serde_json::to_string(info).unwrap();
        self.conn
            .execute("UPDATE info SET json = ?", params![json])
            .unwrap();
    }

    pub fn add_user(&self, username: &str, pw_hash: &str) {
        self.conn
            .execute(
                "INSERT INTO user (user_id, username, pw_hash, signup_time) VALUES (?, ?, ?, ?)",
                params![generate_id(), username, pw_hash, timestamp()],
            )
            .unwrap();
    }

    pub fn query_user_id(&self, username: &str) -> rusqlite::Result<u64> {
        self.conn
            .query_row("SELECT user_id FROM user", [], |r| r.get(0))
    }

    pub fn create_diary_book(&self, name: &str, user_id: u64) {
        let book_id = generate_id();
        self.conn
            .execute(
                "INSERT INTO diary_book (id, name, creation_time) VALUES (?, ?, ?)",
                params![book_id, name, timestamp()],
            )
            .unwrap();
        self.conn
            .execute(
                "INSERT INTO user_diary_book (user_id, diary_book_id) VALUES (?, ?)",
                params![user_id, book_id],
            )
            .unwrap();
    }
}
