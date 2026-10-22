use rusqlite;
use rusqlite::Connection;
use std::path::Path;
use std::time::SystemTime;
use std::{fs, time::UNIX_EPOCH};

use crate::network_ctrl::{PingData, ScanData};

pub struct Central<STATE> {
    _state: STATE,
}


// TODO: whole lotta unwraps... more error handling!
// TODO: pull out the paths to the db init query and the db to something in the config file

pub struct Empty;

pub struct Connected {
    // at this point the database is assumed to be up and functioning properly
    connection: Connection,
}

pub struct Stable {
    // at this point all the queries are prepared & everything is up and running!
    connection: Connection,
    queries: Query,
}

pub struct Query {
    log_scan: String,
    log_ping: String,
    check_auth: String,
    check_open: String,
    door_exists: String,
    user_exists: String,
}

fn load_query_strings() -> Query {
    // should eventually be from the config or wtv
    let log_scan = fs::read_to_string(Path::new("./sql_files/LogScan.sql")).unwrap();
    let log_ping = fs::read_to_string(Path::new("./sql_files/LogPing.sql")).unwrap();
    let check_auth = fs::read_to_string(Path::new("./sql_files/CheckAuth.sql")).unwrap();
    let check_open = fs::read_to_string(Path::new("./sql_files/DoorOpenPublic.sql")).unwrap();
    let door_exists = fs::read_to_string(Path::new("./sql_files/DoorExists.sql")).unwrap();
    let user_exists = fs::read_to_string(Path::new("./sql_files/UserExists.sql")).unwrap();

    Query {
        log_scan,
        check_auth,
        log_ping,
        check_open,
        door_exists,
        user_exists
    }
}

fn load_database_init_query() -> std::io::Result<String> {
    fs::read_to_string(Path::new("./sql_files/CARDatabase.sql"))
}

fn load_init_public_query() -> std::io::Result<String> {
    fs::read_to_string(Path::new("./sql_files/InitPublicRole.sql"))
}

fn load_check_public_init_query() -> std::io::Result<String> {
    fs::read_to_string(Path::new("./sql_files/CheckPublic.sql"))
}

fn current_time_unix() -> String {
    SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
                .to_string()
}

impl Central<Empty> {
    pub fn new() -> Self {
        Central { _state: Empty }
    }

    fn get_connection() -> rusqlite::Result<Connection> {
        let db_path = Path::new("./database.db");
        let conn = rusqlite::Connection::open(db_path)?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        Ok(conn)
    }

    fn init_database(conn: &mut Connection, database_init_query: &String) -> rusqlite::Result<()> {
        conn.execute_batch(database_init_query)?;
        // oooh a constant! rn its 1. a blank database has 0
        conn.pragma_update(None, "user_version", 1)?; 
        Ok(())
    }

    fn init_public(conn: &mut Connection, init_public_query: &String) -> rusqlite::Result<()> {
        conn.execute_batch(init_public_query)?;
        Ok(())
    }

    fn check_database_init(conn: &Connection) -> bool {
        let version = conn.query_one("SELECT user_version FROM pragma_user_version", [], |r| r.get(0));
        match version {
            Ok(1) => true,
            Ok(_) => false,
            Err(e) => {
                eprintln!("Error while reading database user_version {:?}", e);
                return true // this avoid future errors?
            }
        }
    }

    fn check_public_init(conn: &Connection) -> bool {
        let public_exists = conn.query_one(&load_check_public_init_query().unwrap(), [], |r| r.get(0));
        match public_exists {
            Ok(val) => val,
            Err(e) => {
                eprintln!("Error while checking if the public role exists {:?}", e);
                return false
            }
        }
    }

    pub fn connect(self) -> Central<Connected> {
        // TODO: no more unwrap...
        let mut conn = Self::get_connection().unwrap();

        if !Self::check_database_init(&conn) {
            // should probably be something in-between re-init the database and do nothing...
            let _ = Self::init_database(&mut conn, &load_database_init_query().unwrap());
        }

        if !Self::check_public_init(&conn) {
            let _ = Self::init_public(&mut conn, &load_init_public_query().unwrap());
        }


        Central {
            _state: Connected { connection: conn },
        }
    }
}

impl Central<Connected> {
    fn cache_queries(&mut self, query: &Query) -> rusqlite::Result<()> {
        self._state.connection.prepare_cached(&query.log_scan).unwrap();
        self._state.connection.prepare_cached(&query.log_ping).unwrap();
        self._state.connection.prepare_cached(&query.check_auth).unwrap();
        self._state.connection.prepare_cached(&query.check_open).unwrap();
        self._state.connection.prepare_cached(&query.door_exists).unwrap();
        self._state.connection.prepare_cached(&query.user_exists).unwrap();
        Ok(())
    }

    pub fn prepare_queries(mut self) -> Central<Stable> {
        let query = load_query_strings();
        self.cache_queries(&query).unwrap();

        Central {
            _state: Stable {
                connection: self._state.connection,
                queries: query,
            },
        }
    }
}

impl Central<Stable> {
    fn log_ping(&mut self, ping: PingData) {
        let mut command = self
            ._state
            .connection
            .prepare_cached(&self._state.queries.log_ping)
            .unwrap();
        // this is uncomfortably welded to the sql command read in from memory...
        let _ = command.execute([
            ping.door_id,
            current_time_unix(),
        ]);
    }

    fn log_scan(&mut self, scan: ScanData, response: bool) {
        let mut command = self
            ._state
            .connection
            .prepare_cached(&self._state.queries.log_scan)
            .unwrap();
        
        let _ = command.execute([
            scan.door_id,
            scan.user_id,
            current_time_unix(),
            response.to_string(),
        ]);
    }
    
    fn check_user_exists(&mut self, user_id: String) -> bool {
       let mut command = self
                ._state
                .connection
                .prepare_cached(&self._state.queries.user_exists)
                .unwrap();
 
        match command.query_one([user_id], |r| r.get(0)) {
            Ok(val) => val,
            Err(e) => {
                eprintln!("Boolean query DoorExists didn't work: {:?}", e);
                false
            }
        } 
    }

    fn check_door_exists(&mut self, door_id: String) -> bool {
       let mut command = self
                ._state
                .connection
                .prepare_cached(&self._state.queries.door_exists)
                .unwrap();
 
        match command.query_one([door_id], |r| r.get(0)) {
            Ok(val) => val,
            Err(e) => {
                eprintln!("Boolean query DoorExists didn't work: {:?}", e);
                false
            }
        } 
    }

    fn check_auth_query(&mut self, scan: ScanData) -> bool {
        let mut command = self
                ._state
                .connection
                .prepare_cached(&self._state.queries.check_auth)
                .unwrap();
 
        match command.query_one([scan.door_id.clone(), scan.user_id.clone()], |r| r.get(0)) {
            Ok(val) => val,
            Err(e) => {
                eprintln!("Boolean query CheckAuth didn't work: {:?}", e);
                false
            }
        }
    }

    pub fn check_auth(&mut self, scan: ScanData) -> bool {
        if !self.check_door_exists(scan.door_id.clone()) || !self.check_user_exists(scan.user_id.clone()) {
            return false
        }
        let response = self.check_auth_query(scan.clone());
        self.log_scan(scan, response);
        response
    }

    fn check_open_query(&mut self, ping: PingData) -> bool {
        let mut command = self
            ._state
            .connection
            .prepare_cached(&self._state.queries.check_open)
            .unwrap(); // since its in the cache this shouldn't? panic :)

        match command.query_one([ping.door_id.clone()], |r| r.get(0)) {
            Ok(val) => val,
            Err(e) => {
                eprintln!("Boolean query CheckPublic didn't work: {:?}", e);
                false
            }
        }
    }

    pub fn check_open(&mut self, ping: PingData) -> bool {
        if !self.check_door_exists(ping.door_id.clone()) { 
            return false
        }
        let response = self.check_open_query(ping.clone());
        self.log_ping(ping);
        response
    }
}