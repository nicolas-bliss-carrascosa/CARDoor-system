use rusqlite::*;
use std::path::Path;
use std::fs;

use crate::network_ctrl;

struct Central<STATE> {
    _state: STATE
}

struct Empty;

struct Connected {
    connection: Connection,
}

struct Stable<'a> {
    connection: Connection,
    log_scan_query: Statement<'a>,
    log_liveness_query: Statement<'a>,
    check_allowed_in: Statement<'a>,   
}

// TODO: pull out the paths to the db init query and the db to something in the config file

impl Central<Empty> {
    fn get_connection(&self) -> Result<Connection> {
        // TODO: wrap the others
        let db_path = Path::new("./database.db")?;
        rusqlite::Connection::open(db_path)? 
    }

    pub fn connect(self) -> Central<Connected> {
        // TODO: no more unwrap...
        let conn = self.get_connection().unwrap();
        Central {
            _state: Connected {
                connection: conn
            }
        }
    }
}

// impl<'a> CentralSQLState<'a, Connected> {

// }

// fn prep_log_scan(connection: &Connection) -> Result<Statement> {
//     let log_scan_query = fs::read_to_string(Path::new("./LogScan.sql"))?;
//     connection.prepare(&log_scan_query)
// }

// fn check_database() -> bool {
//     // TODO: check if the database exists
//     todo!()
// }



// pub fn hello() {
//     print!("hello!")
// }
