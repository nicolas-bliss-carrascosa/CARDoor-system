use sqlite;
use std::path::Path;
use std::fs;

pub fn main() {
    let db_path = Path::new("./database.db");
    let init_path = Path::new("./CARDatabase.sql");
    
    let connection = sqlite::open(db_path).unwrap();
    let init_database_query = fs::read_to_string(init_path).unwrap();
    let res = connection.execute(init_database_query);
    println!("{:?}", res);
}

pub fn hello() {
    print!("hello!")
}