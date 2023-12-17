-- Flavor: SQLite3

CREATE TABLE IF NOT EXISTS user
(
    -- unchangeable internal identifier
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    -- changeable identifier
    username      TEXT    NOT NULL UNIQUE,
    password_hash TEXT    NOT NULL,
    -- this varies for each user
    password_salt BLOB    NOT NULL,
    name          TEXT,
    email         TEXT,
    -- 0: unknown, 1: male, 2: female, 3: other
    gender_code DEFAULT 0 NOT NULL,
    -- not null if `gender_code` is "other"
    gender_other,
    -- UNIX timestamp in seconds
    signup_time   INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS info
(
    json TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS diary
(
    -- always use date integer as the id
    id            INTEGER NOT NULL PRIMARY KEY,
    content       TEXT    NOT NULL,
    -- UNIX timestamp in seconds
    creation_time INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS diary_book
(
    id            INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    name          TEXT    NOT NULL,
    -- UNIX timestamp in seconds
    creation_time INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS mapping_diary_book_diary_entry
(
    book_id  INTEGER NOT NULL,
    diary_id INTEGER NOT NULL UNIQUE,
    FOREIGN KEY (book_id) REFERENCES diary_book (id),
    FOREIGN KEY (diary_id) REFERENCES diary (id)
);

CREATE TABLE IF NOT EXISTS mapping_user_diary_book
(
    user_id INTEGER NOT NULL,
    book_id INTEGER NOT NULL UNIQUE,
    FOREIGN KEY (book_id) REFERENCES diary_book (id),
    FOREIGN KEY (user_id) REFERENCES user (id)
);
