pub mod schema;

use rusqlite::{Connection, Result};
use std::path::PathBuf;

pub fn get_db_path() -> PathBuf {
    let mut path = std::env::current_dir().unwrap();
    path.push("data");
    std::fs::create_dir_all(&path).unwrap();
    path.push("app.db");
    path
}

pub fn init_db() -> Result<Connection> {
    let db_path = get_db_path();
    let conn = Connection::open(db_path)?;
    
    // 初始化所有表
    schema::create_tables(&conn)?;
    
    Ok(conn)
}

pub fn get_connection() -> Result<Connection> {
    let db_path = get_db_path();
    Connection::open(db_path)
}
