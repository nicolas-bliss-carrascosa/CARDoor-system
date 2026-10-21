use rusqlite;
use rusqlite::Connection;
use rusqlite::fallible_iterator::Scan;
use std::path::Path;
use std::time::SystemTime;
use std::{fs, time::UNIX_EPOCH};

use crate::network_ctrl::{PingData, Response, ScanData};

struct Central<STATE> {
    _state: STATE,
}

struct Empty;

struct Connected {
    // at this point the database is assumed to be up and functioning properly
    connection: Connection,
}

struct Stable {
    // at this point all the queries are prepared & everything is up and running!
    connection: Connection,
    queries: Query,
}

struct Query {
    log_scan: String,
    log_ping: String,
    check_auth: String,
}

// TODO: pull out the paths to the db init query and the db to something in the config file
// TODO: remove all the .unwrap's...

fn load_query_strings() -> std::io::Result<Query> {
    // should eventually be from the config or wtv
    let log_scan = fs::read_to_string(Path::new("./LogScan.sql"))?;
    let check_auth = fs::read_to_string(Path::new("./AuthQuery.sql"))?;
    let log_ping = fs::read_to_string(Path::new("./LogPing.sql"))?;

    Ok(Query {
        log_scan,
        check_auth,
        log_ping,
    })
}

fn load_database_init_query() -> std::io::Result<String> {
    fs::read_to_string(Path::new("./CARDatabase.sql"))
}

impl Central<Empty> {
    fn get_connection() -> rusqlite::Result<Connection> {
        let db_path = Path::new("./database.db");
        rusqlite::Connection::open(db_path)
    }

    fn init_database(conn: &mut Connection, database_init_query: &String) -> rusqlite::Result<()> {
        conn.execute_batch(database_init_query)?;
        Ok(())
    }

    fn check_database(conn: &Connection) -> bool {
        todo!()
    }

    pub fn connect(self) -> Central<Connected> {
        // TODO: no more unwrap...
        let mut conn = Self::get_connection().unwrap();

        if !Self::check_database(&conn) {
            // should probably be something in-between re-init the database and do nothing...
            Self::init_database(&mut conn, &load_database_init_query().unwrap());
        }

        Central {
            _state: Connected { connection: conn },
        }
    }
}

impl Central<Connected> {
    fn cache_queries(&mut self, query: &Query) -> rusqlite::Result<()> {
        self._state.connection.prepare_cached(&query.log_scan)?;
        self._state.connection.prepare_cached(&query.log_ping)?;
        self._state.connection.prepare_cached(&query.check_auth)?;
        Ok(())
    }

    pub fn prepare_queries(mut self) -> Central<Stable> {
        // TODO: the point of this function is to be sensible about handling these error cases, so do better than unwrap!
        let query = load_query_strings().unwrap();
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
    pub fn log_ping(&mut self, ping: PingData) {
        let mut command = self
            ._state
            .connection
            .prepare_cached(&self._state.queries.log_ping)
            .unwrap();
        // this is uncomfortably welded to the sql command read in from memory...
        command.execute([
            ping.door_id,
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
                .to_string(),
        ]);
    }

    pub fn log_scan(&mut self, scan: ScanData, response: bool) {
        let mut command = self
            ._state
            .connection
            .prepare_cached(&self._state.queries.log_scan)
            .unwrap();
        // response is an enum, so by storing it here we assume it will never change?
        command.execute([
            scan.door_id,
            scan.user_id,
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
                .to_string(),
            response.to_string(),
        ]);
    }

    pub fn check_auth(&mut self, scan: ScanData) {
        let mut command = self
            ._state
            .connection
            .prepare_cached(&self._state.queries.check_auth)
            .unwrap();
        let valid_ranges = command.query_map([scan.door_id, scan.user_id], |row| row.get(0)).unwrap();
        // process the valid ranges & current time
    }
}
