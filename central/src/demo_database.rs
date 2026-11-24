//! Smoke/integration tests for the `Central` database API.
//!
//! Usage (in main.rs):
//!
//!     mod db_tests;
//!
//!     fn main() {
//!         if std::env::args().any(|a| a == "--db-test") {
//!             std::process::exit(if db_tests::run_db_tests() { 0 } else { 1 });
//!         }
//!         // ... normal startup ...
//!     }
//!
//! Run with:  cargo run -- --db-test      (from the same directory you normally run from)
//!
//! How it works
//! ------------
//! `Central` hard-codes `../db/...` relative to the working directory, so this harness builds a
//! throw-away sandbox in the OS temp dir:
//!
//!     <tmp>/car_db_test_<pid>/db/    <- copies of every *.sql found in your real ../db
//!     <tmp>/car_db_test_<pid>/run/   <- the cwd while tests run, so ../db resolves to the above
//!
//! Your real database.db is never touched. The sandbox is deleted afterwards unless a test failed
//! (then its path is printed so you can poke at it).
//!
//! Only the public API is used for the behaviour under test (`connect`, `prepare_queries`,
//! `check_auth`, `check_open`); a separate raw rusqlite connection seeds data and inspects the
//! resulting tables.

use std::fmt::Debug;
use std::panic::{self, AssertUnwindSafe};
use std::path::PathBuf;
use std::{env, fs};

use rusqlite::Connection;

// ---- ADJUST THESE TWO LINES to match your project ------------------------------------------
use crate::sql_ctrl::{Central, Stable}; // wherever mod.rs lives
use crate::network_ctrl::{PingData, ScanData};

fn scan(door: &str, user: &str) -> ScanData {
    ScanData { door_id: door.to_string(), user_id: user.to_string() }
}
fn ping(door: &str) -> PingData {
    PingData { door_id: door.to_string() }
}
// ---------------------------------------------------------------------------------------------

/// The time-of-day expression your SQL files use. If you change the queries to
/// `strftime('%H:%M:%S','now','localtime')`, flip this to true so the test windows line up.
const USE_LOCALTIME: bool = false;

// ================================================================== tiny test framework

#[derive(Default)]
struct Report {
    passed: u32,
    failed: u32,
}

impl Report {
    fn pass(&mut self, name: &str) {
        self.passed += 1;
        println!("  ok    {name}");
    }
    fn fail(&mut self, name: &str, why: &str) {
        self.failed += 1;
        println!("  FAIL  {name}: {why}");
    }
    fn check(&mut self, name: &str, ok: bool, why: &str) {
        if ok { self.pass(name) } else { self.fail(name, why) }
    }
    fn eq<T: PartialEq + Debug>(&mut self, name: &str, got: T, want: T) {
        if got == want {
            self.pass(name)
        } else {
            self.fail(name, &format!("got {got:?}, want {want:?}"))
        }
    }
}

fn section(title: &str) {
    println!("\n== {title}");
}

/// Run `f`, turning a panic into a recorded FAIL so the remaining tests still run.
fn guard<T>(r: &mut Report, name: &str, f: impl FnOnce() -> T) -> Option<T> {
    match panic::catch_unwind(AssertUnwindSafe(f)) {
        Ok(v) => Some(v),
        Err(_) => {
            r.fail(name, "panicked (see message above)");
            None
        }
    }
}

fn count(conn: &Connection, sql: &str) -> i64 {
    conn.query_row(sql, [], |row| row.get(0))
        .unwrap_or_else(|e| panic!("test query failed: {sql}\n  {e}"))
}

fn short(s: &str) -> String {
    format!("{s:?}").chars().take(30).collect()
}

// ====================================================================== sandbox

struct Sandbox {
    root: PathBuf,
    orig_cwd: PathBuf,
    keep: bool,
}

impl Sandbox {
    fn create() -> std::io::Result<Self> {
        let orig_cwd = env::current_dir()?;
        let src = orig_cwd.join("../db");
        let root = env::temp_dir().join(format!("car_db_test_{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("db"))?;
        fs::create_dir_all(root.join("run"))?;

        let mut copied = 0;
        for entry in fs::read_dir(&src)? {
            let path = entry?.path();
            if path.extension().and_then(|e| e.to_str()) == Some("sql") {
                fs::copy(&path, root.join("db").join(path.file_name().unwrap()))?;
                copied += 1;
            }
        }
        println!("sandbox: {} ({copied} .sql files copied from {})", root.display(), src.display());

        env::set_current_dir(root.join("run"))?;
        Ok(Sandbox { root, orig_cwd, keep: false })
    }

    fn db_path(&self) -> PathBuf {
        self.root.join("db").join("database.db")
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = env::set_current_dir(&self.orig_cwd);
        if self.keep {
            println!("\n(sandbox kept for inspection: {})", self.root.display());
        } else {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}

// ================================================================= test data

/// Current time-of-day, shifted by an SQLite modifier like "-1 hours".
fn tod(conn: &Connection, modifier: &str) -> String {
    let sql = if USE_LOCALTIME {
        "SELECT strftime('%H:%M:%S','now',?1,'localtime')"
    } else {
        "SELECT strftime('%H:%M:%S','now',?1)"
    };
    conn.query_row(sql, [modifier], |r| r.get(0)).unwrap()
}

/// Time windows relative to "now", so the tests pass at any time of day.
struct Windows {
    open: (String, String),      // contains now
    closed: (String, String),    // does not contain now
    overnight: (String, String), // start > end (wraps midnight) and contains now
}

fn windows(conn: &Connection) -> Windows {
    let now = tod(conn, "+0 hours");
    let (s, e) = (tod(conn, "-1 hours"), tod(conn, "+1 hours"));
    let open = if s <= e {
        (s, e)
    } else {
        println!("note: within an hour of midnight, using a full-day 'open' window");
        ("00:00:00".to_string(), "23:59:59".to_string())
    };
    let closed = if now.as_str() < "12:00:00" {
        ("13:00:00".to_string(), "14:00:00".to_string())
    } else {
        ("01:00:00".to_string(), "02:00:00".to_string())
    };
    let overnight = (tod(conn, "-1 hours"), tod(conn, "-2 hours"));
    println!("now={now}  open={open:?}  closed={closed:?}  overnight={overnight:?}");
    Windows { open, closed, overnight }
}

/// Layout:
///   users:  1 Alice (Employee)   2 Bob (Contractor)   3 Carol (no roles)   4 Dan (Overnight)
///   roles:  1 Public (made by connect())  2 Employee  3 Contractor  4 Overnight
///   doors:  1,2 HQ   3,4 Warehouse   5 Atrium   6,7 Annex   8 Depot
fn seed_sql(w: &Windows) -> String {
    format!(
        "
        INSERT INTO Users (UserId, Name, DateIssued) VALUES
            (1,'Alice',0), (2,'Bob',0), (3,'Carol',0), (4,'Dan',0);
        INSERT INTO Roles (RoleId, description) VALUES
            (2,'Employee'), (3,'Contractor'), (4,'Overnight');
        INSERT INTO UserXRole (UserId, RoleId, DateIssued) VALUES
            (1,2,0), (2,3,0), (4,4,0);
        INSERT INTO Facilities (FacilityId, Name) VALUES
            (1,'HQ'), (2,'Warehouse'), (3,'Atrium'), (4,'Annex'), (5,'Depot');
        INSERT INTO Doors (DoorId, FacilityId, Name, ExpectedState) VALUES
            (1,1,'HQ Front',0), (2,1,'HQ Server Room',0),
            (3,2,'WH Dock',0),  (4,2,'WH Office',0),
            (5,3,'Atrium Door',0),
            (6,4,'Annex Door',0), (7,4,'Annex Side Door',0),
            (8,5,'Depot Door',0);

        INSERT INTO FacilityXRole (FacilityId, RoleId, DateIssued) VALUES
            (1,2,0), (2,3,0), (3,1,0), (4,1,0), (5,4,0);

        INSERT INTO FacilityXRoleXTimeRange (RoleId, FacilityId, StartTime, EndTime) VALUES
            (2,1,'{os}','{oe}'),   -- Employee: HQ, open now
            (3,2,'{cs}','{ce}'),   -- Contractor: Warehouse, window NOT containing now
            (1,3,'{os}','{oe}'),   -- Public: Atrium, open now
            (1,4,'{cs}','{ce}'),   -- Public: Annex, closed now
            (4,5,'{ns}','{ne}');   -- Overnight: Depot, wrapping window containing now

        -- exception layer
        INSERT INTO DoorXRole (RoleId, DoorId, Allowed, DateIssued) VALUES
            (3,3,1,0),   -- Contractor allowed at WH Dock
            (2,2,0,0),   -- Employee DENIED at HQ Server Room (despite facility access)
            (1,7,1,0);   -- Public allowed at Annex Side Door
        INSERT INTO DoorXRoleXTimeRange (RoleId, DoorId, StartTime, EndTime) VALUES
            (3,3,'{os}','{oe}'),
            (1,7,'{os}','{oe}');
        ",
        os = w.open.0,
        oe = w.open.1,
        cs = w.closed.0,
        ce = w.closed.1,
        ns = w.overnight.0,
        ne = w.overnight.1,
    )
}

// ================================================================ assertions on the API

fn expect_auth(r: &mut Report, db: &mut Central<Stable>, name: &str, door: &str, user: &str, want: bool) {
    if let Some(got) = guard(r, name, || db.check_auth(scan(door, user))) {
        r.eq(name, got, want);
    }
}

fn expect_open(r: &mut Report, db: &mut Central<Stable>, name: &str, door: &str, want: bool) {
    if let Some(got) = guard(r, name, || db.check_open(ping(door))) {
        r.eq(name, got, want);
    }
}

// ==================================================================== the tests

/// Returns true if everything passed.
pub fn test() {
    let mut r = Report::default();
    let mut sb = match Sandbox::create() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("could not build sandbox (is ../db reachable from the cwd?): {e}");
            return;
        }
    };
    run_all(&mut r, &sb);

    println!("\n{} passed, {} failed", r.passed, r.failed);
    sb.keep = r.failed > 0;
}

fn run_all(r: &mut Report, sb: &Sandbox) {
    // ---------------------------------------------------------------- 1. init
    section("1. init on a fresh database (and re-connect)");
    for i in 1..=2 {
        let name = format!("connect() #{i} doesn't panic");
        if guard(r, &name, || {
            let _ = Central::new().connect();
        })
        .is_some()
        {
            r.pass(&name);
        }
    }

    let raw = match Connection::open(sb.db_path()) {
        Ok(c) => c,
        Err(e) => return r.fail("open sandbox db", &e.to_string()),
    };
    raw.pragma_update(None, "foreign_keys", "ON").unwrap();

    r.eq("user_version == 1", count(&raw, "SELECT user_version FROM pragma_user_version"), 1);

    let expected_tables = [
        "Users", "Roles", "Facilities", "Doors", "UserXRole", "FacilityXRole", "DoorXRole",
        "FacilityXRoleXTimeRange", "DoorXRoleXTimeRange", "Logs", "LivenessLog",
    ];
    let missing: Vec<&str> = expected_tables
        .iter()
        .copied()
        .filter(|t| {
            count(&raw, &format!("SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='{t}'")) == 0
        })
        .collect();
    r.check(
        "all tables created",
        missing.is_empty(),
        &format!("missing {missing:?} -- CARDatabase.sql failed part-way and connect() swallows the error"),
    );
    if !missing.is_empty() {
        println!("\nschema incomplete, aborting the remaining phases");
        return;
    }
    r.eq("public role (id 1) exists exactly once after 2 connects", count(&raw, "SELECT COUNT(*) FROM Roles WHERE RoleId = 1"), 1);

    // --------------------------------------------------------------- 2. seed
    section("2. seed test data (raw connection)");
    let w = windows(&raw);
    if let Err(e) = raw.execute_batch(&seed_sql(&w)) {
        return r.fail("seed data", &format!("{e} (schema doesn't match what this harness expects)"));
    }
    r.pass("seed data inserted (foreign keys ON)");

    // ----------------------------------------------------------- 3. prepare
    section("3. prepare queries");
    let built = panic::catch_unwind(|| Central::new().connect().prepare_queries());
    let mut db = match built {
        Ok(db) => {
            r.pass("prepare_queries() doesn't panic");
            db
        }
        Err(_) => {
            r.fail(
                "prepare_queries() doesn't panic",
                "panicked -- a .sql file doesn't compile against the schema (the panic message above names the error)",
            );
            return;
        }
    };

    // ---------------------------------------------------- 4. auth decisions
    section("4. check_auth: decisions");
    expect_auth(r, &mut db, "Alice @ HQ Front: role window open -> granted", "1", "1", true);
    expect_auth(r, &mut db, "Alice @ HQ Server Room: explicit deny beats facility access", "2", "1", false);
    expect_auth(r, &mut db, "Bob @ WH Dock: door exception grants despite closed facility window", "3", "2", true);
    expect_auth(r, &mut db, "Bob @ WH Office: facility window closed -> denied", "4", "2", false);
    expect_auth(r, &mut db, "Carol @ HQ Front: user with no roles -> denied", "1", "3", false);
    expect_auth(r, &mut db, "unknown door -> denied", "999", "1", false);
    expect_auth(r, &mut db, "unknown user -> denied", "1", "999", false);

    // ------------------------------------------------------- 5. audit log
    section("5. check_auth: audit log");
    r.eq("one Logs row per scan of a known user+door (5)", count(&raw, "SELECT COUNT(*) FROM Logs"), 5);
    r.eq("2 granted scans logged as granted", count(&raw, "SELECT COUNT(*) FROM Logs WHERE Result IN (1,'true')"), 2);
    r.eq("3 denied scans logged as denied", count(&raw, "SELECT COUNT(*) FROM Logs WHERE Result IN (0,'false')"), 3);

    let before = count(&raw, "SELECT COUNT(*) FROM Logs");
    guard(r, "rapid repeat scans", || {
        for _ in 0..3 {
            db.check_auth(scan("1", "1"));
        }
    });
    r.eq(
        "3 scans within the same second -> 3 log rows (PK is DoorId+UserId+Date at 1s resolution)",
        count(&raw, "SELECT COUNT(*) FROM Logs") - before,
        3,
    );

    // ------------------------------------------------ 6. hostile / malformed
    // section("6. check_auth: malformed and hostile ids");
    // let logs_before = count(&raw, "SELECT COUNT(*) FROM Logs");
    // let long = "9".repeat(10_000);
    // let weird: Vec<&str> = vec![
    //     "", " ", "abc", "-1", "0", "99999999999999999999", "1 OR 1=1", "1; DROP TABLE Users;--", "' OR ''='",
    //     "\u{ff11}", /* fullwidth 1 */ "NULL", "1\0", long.as_str(),
    // ];
    // for wd in &weird {
    //     expect_auth(r, &mut db, &format!("door id {} -> denied, no panic", short(wd)), wd, "1", false);
    //     expect_auth(r, &mut db, &format!("user id {} -> denied, no panic", short(wd)), "1", wd, false);
    // }
    // r.eq("Users table intact after injection attempts", count(&raw, "SELECT COUNT(*) FROM Users"), 4);
    // r.eq("Doors table intact after injection attempts", count(&raw, "SELECT COUNT(*) FROM Doors"), 8);
    // println!(
    //     "  info  {} log rows written for {} scans of unknown ids (0 = probing leaves no audit trail)",
    //     count(&raw, "SELECT COUNT(*) FROM Logs") - logs_before,
    //     weird.len() * 2
    // );

    // for alias in ["1.0", "+1", "1 ", "01"] {
    //     expect_auth(
    //         r,
    //         &mut db,
    //         &format!("[hardening] door id {} must not alias door 1", short(alias)),
    //         alias,
    //         "1",
    //         false,
    //     );
    // }

    // ------------------------------------------------------ 7. public pings
    section("7. check_open (public door state)");
    expect_open(r, &mut db, "Atrium door: public facility window open -> true", "5", true);
    expect_open(r, &mut db, "Annex door: public facility window closed -> false", "6", false);
    expect_open(r, &mut db, "Annex side door: door-level public exception open -> true", "7", true);
    expect_open(r, &mut db, "HQ Front: no public rule at all -> false", "1", false);
    expect_open(r, &mut db, "unknown door -> false", "999", false);
    expect_open(r, &mut db, "garbage door id -> false", "1 OR 1=1", false);

    // ---------------------------------------------------- 8. ping side-effects
    section("8. check_open: liveness log and ExpectedState");
    r.eq("LivenessLog: one row per ping of a known door (4)", count(&raw, "SELECT COUNT(*) FROM LivenessLog"), 4);
    println!(
        "  info  Sent values: {}",
        raw.query_row("SELECT group_concat(DISTINCT Sent || ' (' || typeof(Sent) || ')') FROM LivenessLog", [], |r| r.get::<_, String>(0))
            .unwrap_or_default()
    );
    let norm = |door: i64| -> i64 {
        count(
            &raw,
            &format!("SELECT CASE WHEN ExpectedState IN (1,'true') THEN 1 ELSE 0 END FROM Doors WHERE DoorId = {door}"),
        )
    };
    r.check(
        "ExpectedState differs between an open door (5) and a closed one (6)",
        norm(5) != norm(6),
        &format!("both are {}", norm(5)),
    );

    let before = count(&raw, "SELECT COUNT(*) FROM LivenessLog");
    guard(r, "rapid repeat pings", || {
        for _ in 0..3 {
            db.check_open(ping("5"));
        }
    });
    r.eq(
        "3 pings within the same second -> 3 liveness rows (PK is DoorId+Date at 1s resolution)",
        count(&raw, "SELECT COUNT(*) FROM LivenessLog") - before,
        3,
    );
}