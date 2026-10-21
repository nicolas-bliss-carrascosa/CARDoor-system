// Start the server first, then: cargo run --bin demo_client

use std::{
    io::{BufRead, BufReader, Write},
    net::TcpStream,
    sync::atomic::{AtomicUsize, Ordering},
    thread,
    time::{Duration, Instant},
};

const ADDR: &str = "127.0.0.1:7000";
static FAILS: AtomicUsize = AtomicUsize::new(0);

struct Client {
    w: TcpStream,
    r: BufReader<TcpStream>,
}

impl Client {
    fn connect() -> Client {
        let s = TcpStream::connect(ADDR).expect("connect failed - is the server running?");
        s.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
        Client { w: s.try_clone().unwrap(), r: BufReader::new(s) }
    }

    fn send(&mut self, raw: &str) {
        self.w.write_all(raw.as_bytes()).unwrap();
    }

    fn recv(&mut self) -> String {
        let mut line = String::new();
        match self.r.read_line(&mut line) {
            Ok(0) => "<closed>".into(),
            Ok(_) => line.trim().to_string(),
            Err(e) => format!("<error: {e}>"),
        }
    }
    
    fn ask(&mut self, line: &str) -> String {
        self.send(&format!("{line}\n"));
        self.recv()
    }
}

fn check(name: &str, got: &str, expected: &str) {
    if got == expected {
        println!("  PASS  {name}");
    } else {
        println!("  FAIL  {name}: expected {expected:?}, got {got:?}");
        FAILS.fetch_add(1, Ordering::Relaxed);
    }
}

fn main() {
    println!("1. Persistent connection: several requests on ONE socket");
    let mut c = Client::connect();
    check("SCAN alice", &c.ask("SCAN door1 alice"), "GRANTED");
    check("SCAN mallory", &c.ask("SCAN door1 mallory"), "DENIED");

    println!("2. Command split across packets, with a pause longer than the server's read timeout");
    c.send("SCAN door1 al");
    thread::sleep(Duration::from_millis(700));
    c.send("ice\n");
    check("split SCAN alice", &c.recv(), "GRANTED");

    println!("3. Two commands in a single write (pipelining)");
    c.send("PING door1\nSCAN door1 alice\n");
    check("first reply", &c.recv(), "DENIED");
    check("second reply", &c.recv(), "GRANTED");

    println!("4. Malformed input");
    check("unknown command", &c.ask("OPEN door1"), "ERR unknown command");
    check("SCAN missing user", &c.ask("SCAN door1"), "ERR SCAN requires a door id and a user id");
    check("PING missing door", &c.ask("PING"), "ERR PING requires only a door id");

    println!("5. Oversized line (no newline) gets rejected and the connection closed");
    let mut big = Client::connect();
    big.send(&"A".repeat(300));
    check("too long", &big.recv(), "ERR line too long");
    check("then closed", &big.recv(), "<closed>");

    println!("6. Many clients at once: 10 threads x 20 requests each");
    let start = Instant::now();
    let handles: Vec<_> = (0..10)
        .map(|_| {
            thread::spawn(|| {
                let mut c = Client::connect();
                let mut bad = 0;
                for i in 0..20 {
                    let (user, want) = if i % 2 == 0 { ("alice", "GRANTED") } else { ("mallory", "DENIED") };
                    if c.ask(&format!("SCAN door1 {user}")) != want {
                        bad += 1;
                    }
                }
                bad
            })
        })
        .collect();
    let bad: usize = handles.into_iter().map(|h| h.join().unwrap()).sum();
    check("200 concurrent replies correct", &bad.to_string(), "0");
    println!("     ({} ms)", start.elapsed().as_millis());

    println!("7. A stalled request doesn't block other clients");
    let slow = thread::spawn(|| {
        let t = Instant::now();
        let reply = Client::connect().ask("SCAN door1 dna");
        (reply, t.elapsed())
    });
    thread::sleep(Duration::from_millis(200));
    let t = Instant::now();
    let fast = Client::connect().ask("PING door1");
    check("PING while another client is stalled", &fast, "DENIED");
    println!("     (answered in {} ms)", t.elapsed().as_millis());
    let (reply, took) = slow.join().unwrap();
    check("stalled client gets timeout", &reply, "ERR system timeout");
    println!("     (timeout after {} ms, expect ~2000)", took.as_millis());

    println!("8. Client disconnects abruptly");
    {
        let mut gone = Client::connect();
        gone.send("SCAN door1 ali"); // partial line, then drop the socket
    }
    thread::sleep(Duration::from_millis(700));
    check("server still healthy", &Client::connect().ask("PING door1"), "DENIED");

    let fails = FAILS.load(Ordering::Relaxed);
    println!("\n{}", if fails == 0 { "ALL PASSED".to_string() } else { format!("{fails} FAILED") });
}