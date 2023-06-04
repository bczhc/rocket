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
            "CREATE TABLE IF NOT EXISTS user
(
    username TEXT NOT NULL PRIMARY KEY,
    pw_hash  TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS info
(
    json TEXT NOT NULL
)",
        )?;

        conn.execute("INSERT INTO info VALUES ('')", [])?;

        Ok(Self { conn })
    }

    pub fn check_existence(&self, username: &str) -> bool {
        let count: u32 = self
            .conn
            .query_row(
                "SELECT COUNT() FROM user WHERE username IS ?",
                params![username],
                |r| r.get(0),
            )
            .unwrap();
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
                "INSERT INTO user (username, pw_hash) VALUES (?, ?)",
                params![username, pw_hash],
            )
            .unwrap();
    }
}
