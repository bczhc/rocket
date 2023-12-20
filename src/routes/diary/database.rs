use crate::routes::diary::timestamp;
use crate::routes::diary::user::{Gender, UserProfile};
use crate::security::hash_password;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// TODO: return SQLite error and treat as a "server internal error"

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

        conn.execute_batch(include_str!("./schema.sql"))?;

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

    pub fn verify_password(&self, username: &str, password: &str) -> bool {
        if !self.check_existence(username) {
            return false;
        }
        let (hash, salt): (String, String) = self
            .conn
            .query_row(
                "SELECT password_hash, password_salt FROM user WHERE username == ?",
                params![username],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        hash_password(password, salt.as_bytes()) == hash
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

    pub fn add_user(&self, username: &str, pw_hash: &str, salt: &str) {
        self.conn
            .execute(
                "INSERT INTO user (username, password_hash, password_salt, signup_time) VALUES (?, ?, ?, ?)",
                params![username, pw_hash, salt, timestamp()],
            )
            .unwrap();
    }

    pub fn query_user_id(&self, username: &str) -> Option<u64> {
        self.conn
            .query_row(
                "SELECT id FROM user WHERE username IS ?",
                params![username],
                |r| r.get(0),
            )
            .ok()
    }

    pub fn query_user_profile(&self, id: u64) -> Option<UserProfile> {
        let user_profile = self.conn.query_row(
            "SELECT signup_time, name, email, username, gender_code, gender_other FROM user WHERE id IS ?",
            params![id],
            |r| {
                let gender_code: u8 = r.get(4)?;
                let gender_other: Option<String> = r.get(5)?;

                Ok(UserProfile {
                    signup_time: r.get(0)?,
                    name: r.get(1)?,
                    email: r.get(2)?,
                    username: r.get(3)?,
                    gender: Gender::from_db_int(gender_code, gender_other)
                })
            },
        );
        user_profile.ok()
    }

    pub fn update_user_profile(&self, uid: u64, new: &UserProfile) {
        let gender_int = new.gender.to_db_int();

        self.conn
            .execute(
                "UPDATE user
SET username     = ?,
    name         = ?,
    email        = ?,
    gender_code  = ?,
    gender_other = ?
WHERE id = ?",
                params![
                    new.username,
                    new.name,
                    new.email,
                    gender_int.0,
                    gender_int.1,
                    uid,
                ],
            )
            .unwrap();
    }

    pub fn create_diary_book(&self, name: &str, user_id: u64) {
        todo!();
        self.conn
            .execute(
                "INSERT INTO diary_book (name, creation_time) VALUES (?, ?, ?)",
                params![name, timestamp()],
            )
            .unwrap();
        self.conn
            .execute(
                "INSERT INTO user_diary_book (user_id, diary_book_id) VALUES (?, ?)",
                params![user_id, 0],
            )
            .unwrap();
    }
}
