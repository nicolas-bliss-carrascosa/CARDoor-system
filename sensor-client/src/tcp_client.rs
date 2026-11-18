use std::{
    io::{self, BufRead, BufReader, Write},
    net::{SocketAddr, TcpStream},
    sync::mpsc,
    thread,
    time::Duration,
};

pub const ADDR: &str = "127.0.0.1:7000";

pub enum TcpCommand {
    Connect { door_id: String },
    Scan { door_id: String, user_id: String },
}

pub enum TcpEvent {
    Status(String),
    Response(String),
}

pub struct Client {
    w: TcpStream,
    r: BufReader<TcpStream>,
}

impl Client {
    pub fn connect() -> io::Result<Self> {
        let addr: SocketAddr = ADDR
            .parse()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
        let s = TcpStream::connect_timeout(&addr, Duration::from_secs(3))?;
        s.set_read_timeout(Some(Duration::from_secs(5)))?;
        s.set_write_timeout(Some(Duration::from_secs(5)))?;
        let w = s.try_clone()?;
        Ok(Client {
            w,
            r: BufReader::new(s),
        })
    }

    pub fn send(&mut self, raw: &str) -> io::Result<()> {
        self.w.write_all(raw.as_bytes())?;
        self.w.flush()
    }

    pub fn recv(&mut self) -> io::Result<String> {
        let mut line = String::new();
        let bytes = self.r.read_line(&mut line)?;
        if bytes == 0 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "server closed connection",
            ));
        }
        Ok(line.trim().to_string())
    }

    pub fn ask(&mut self, line: &str) -> io::Result<String> {
        self.send(&format!("{line}\n"))?;
        self.recv()
    }

    pub fn ping(&mut self, door_id: &str) -> io::Result<String> {
        let d_id = sanitize_token(door_id, "door_1");
        self.ask(&format!("PING {d_id}"))
    }

    pub fn scan(&mut self, door_id: &str, user_id: &str) -> io::Result<String> {
        let d_id = sanitize_token(door_id, "door_1");
        let u_id = user_id.trim();
        self.ask(&format!("SCAN {d_id} {u_id}"))
    }
}

pub fn sanitize_token(raw: &str, fallback: &str) -> String {
    raw.split_whitespace().next().unwrap_or(fallback).to_string()
}

fn try_connect_and_ping(door_id: &str, event_tx: &mpsc::Sender<TcpEvent>) -> Option<Client> {
    let _ = event_tx.send(TcpEvent::Status(format!("Connecting to {ADDR}...")));
    match Client::connect() {
        Ok(mut client) => {
            let _ = event_tx.send(TcpEvent::Status(format!("Connected to {ADDR}")));
            match client.ping(door_id) {
                Ok(reply) => {
                    let _ = event_tx.send(TcpEvent::Response(reply));
                    Some(client)
                }
                Err(err) => {
                    let _ = event_tx.send(TcpEvent::Status(format!("Disconnected: {err}")));
                    let _ = event_tx.send(TcpEvent::Response(format!("PING failed: {err}")));
                    None
                }
            }
        }
        Err(err) => {
            let _ = event_tx.send(TcpEvent::Status(format!("Connection failed: {err}")));
            None
        }
    }
}

pub fn spawn_tcp_worker(
    initial_door_id: String,
) -> (mpsc::Sender<TcpCommand>, mpsc::Receiver<TcpEvent>) {
    let (cmd_tx, cmd_rx) = mpsc::channel::<TcpCommand>();
    let (event_tx, event_rx) = mpsc::channel::<TcpEvent>();

    thread::spawn(move || {
        let mut client = try_connect_and_ping(&initial_door_id, &event_tx);

        while let Ok(cmd) = cmd_rx.recv() {
            match cmd {
                TcpCommand::Connect { door_id } => {
                    client = None;
                    client = try_connect_and_ping(&door_id, &event_tx);
                }
                TcpCommand::Scan { door_id, user_id } => {
                    if client.is_none() {
                        client = try_connect_and_ping(&door_id, &event_tx);
                    }

                    if let Some(ref mut active) = client {
                        match active.scan(&door_id, &user_id) {
                            Ok(reply) => {
                                let _ = event_tx.send(TcpEvent::Response(reply));
                            }
                            Err(err) => {
                                let _ = event_tx.send(TcpEvent::Status(format!(
                                    "Disconnected: {err}"
                                )));
                                let _ = event_tx.send(TcpEvent::Response(format!(
                                    "SCAN failed: {err}"
                                )));
                                client = None;
                            }
                        }
                    } else {
                        let _ = event_tx.send(TcpEvent::Response(
                            "Not connected — click Retry Connection".into(),
                        ));
                    }
                }
            }
        }
    });

    (cmd_tx, event_rx)
}