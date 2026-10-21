use std::{
    io::{BufRead, BufReader, Error, ErrorKind, Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        mpsc::{self, channel, Receiver, RecvTimeoutError, Sender},
        Arc,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use crate::network_ctrl::{NetEvent, Request, Response, PingData, ScanData};

const MAX_LINE: usize = 256;
const MAX_CLIENTS: usize = 32;
const READ_POLL: Duration = Duration::from_millis(500);
const REPLY_TIMEOUT: Duration = Duration::from_secs(2);

pub struct NetworkHandle {
    pub events: Receiver<NetEvent>,
    pub shutdown: Arc<AtomicBool>,
    pub acceptor: Option<JoinHandle<()>>,
}

impl NetworkHandle {
    fn stop(&mut self) {
        self.shutdown.store(true, Ordering::Relaxed);
        if let Some(h) = self.acceptor.take() {
            let _ = h.join();
        }
    }
}

impl Drop for NetworkHandle {
    fn drop(&mut self) {
        self.stop();
    }
}

/**
 * The interfacing function. Call this at main() and everything will be fine <3.
 */
pub fn start(addr: &str) -> Result<NetworkHandle, Error> {
    let listener = TcpListener::bind(addr)?;

    // Finding this gave me a headache. When you have calls coming right after another,
    // without this, you cannot read the next call before resolving the previous call.
    listener.set_nonblocking(true)?;

    let (tx, rx) = mpsc::channel::<NetEvent>();
    let shutdown = Arc::new(AtomicBool::new(false));
    let active = Arc::new(AtomicUsize::new(0));

    // Create the TCP handshake threads
    let acceptor = {
        let shutdown = Arc::clone(&shutdown);
        thread::Builder::new()
            .name("net-accept".into())
            .spawn(move || accept_loop(listener, tx, shutdown, active))?
    };

    // Return a nice struct that compiles all threads
    Ok(NetworkHandle {
        events: rx,
        shutdown: shutdown,
        acceptor: Some(acceptor),
    })
}

/**
 * Accepts or rejects TCP handshakes
 */
fn accept_loop(
    listener: TcpListener,
    tx: Sender<NetEvent>,
    shutdown: Arc<AtomicBool>,
    active: Arc<AtomicUsize>,
) {
    let mut workers: Vec<JoinHandle<()>> = Vec::new();

    // Basically as long as the state machine did not tell me to shutdown
    while !shutdown.load(Ordering::Relaxed) {
        match listener.accept() {
            Ok((stream, peer)) => {
                if active.load(Ordering::Relaxed) >= MAX_CLIENTS {
                    eprintln!("Too many clients. Rejecting the peer {peer}.");
                    continue;
                }
                active.fetch_add(1, Ordering::Relaxed);

                let tx_t = tx.clone();
                let shutdown_t = Arc::clone(&shutdown);
                let active_t = Arc::clone(&active);

                match thread::Builder::new()
                    .name(format!("net-client-{peer}"))
                    .spawn(move || {
                        if let Err(e) = serve_client(stream, peer, tx_t, &shutdown_t) {
                            eprintln!("Client {peer} ended: {e}");
                        }
                        active_t.fetch_sub(1, Ordering::Relaxed);
                    }) {
                    Ok(h) => workers.push(h),
                    Err(e) => {
                        eprintln!("Failed to spawn a worker for {peer} with error: {e}");
                        active.fetch_sub(1, Ordering::Relaxed);
                    }
                }
                workers.retain(|h| !h.is_finished());
            }
            Err(e) if e.kind() == ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(50));
            }
            Err(e) => {
                eprintln!("Error accepting connection. {e}");
                thread::sleep(Duration::from_millis(50));
            }
        }
    }
    for h in workers {
        let _ = h.join();
    }
}

/**
 * This function deals with the communication between
 */
fn serve_client(
    stream: TcpStream,
    peer: SocketAddr,
    tx: Sender<NetEvent>,
    shutdown: &AtomicBool,
) -> Result<(), Error> {
    stream.set_nonblocking(false)?;
    stream.set_read_timeout(Some(READ_POLL))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;

    let mut writer = stream.try_clone()?;
    let mut reader = BufReader::new(stream);

    if !authenticate(&mut reader, &mut writer, shutdown)? {
        writer.write_all(b"ERR auth failed\n")?;
        eprintln!("{peer} has failed authentication. Dropping the connection.");
        return Ok(());
    }

    let mut buf: Vec<u8> = Vec::with_capacity(MAX_LINE);

    while !shutdown.load(Ordering::Relaxed) {
        match (&mut reader)
            .take(MAX_LINE as u64 + 1)
            .read_until(b'\n', &mut buf)
        {
            Ok(0) => return Ok(()), // Nothing left to read.
            Ok(_) => {}
            Err(e) if matches!(e.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {
                if buf.len() > MAX_LINE {
                    writer.write_all(b"ERR line too long\n")?;
                    eprintln!("Line received from {peer} is too long. Discarding the message.");
                    return Ok(());
                }
                continue;
            }
            Err(e) => return Err(e),
        }

        if buf.last() != Some(&b'\n') {
            if buf.len() > MAX_LINE {
                writer.write_all(b"ERR line too long\n")?;
                eprintln!("Line received from {peer} is too long. Discarding the message.");
                return Ok(());
            }
            continue;
        }

        let line = String::from_utf8_lossy(&buf).trim().to_string();
        buf.clear();

        let reply_line = match parse_line(&line) {
            Ok(request) => handle_request(&tx, peer, request),
            Err(msg) => {
                eprintln!("Error parsing the request line. {msg}");
                format!("ERR {msg}\n")
            }
        };
        writer.write_all(reply_line.as_bytes())?;
    }

    Ok(())
}

/**
 * Parse the incoming message and return a command enum
 */
fn parse_line(line: &str) -> Result<Request, &'static str> {
    let mut parts = line.splitn(3, ' ');
    match (parts.next(), parts.next(), parts.next()) {
        (Some("PING"), Some(d_id), None) if !d_id.trim().is_empty() => Ok(Request::Ping(PingData {
            door_id: d_id.trim().to_string(),
        })),
        (Some("PING"), _, _) => Err("PING requires only a door id"),
        (Some("SCAN"), Some(d_id), Some(u_id))
            if !d_id.trim().is_empty() && !u_id.trim().is_empty() =>
        {
            Ok(Request::Scan(ScanData {
                door_id: d_id.trim().to_string(),
                user_id: u_id.trim().to_string(),
            }))
        }
        (Some("SCAN"), _, _) => Err("SCAN requires a door id and a user id"),
        _ => Err("unknown command"),
    }
}

/**
 * Dummy to hold place for now <3
 * TODO: Fix this
 */
fn authenticate<R: Read, W: Write>(
    _reader: &mut R,
    _writer: &mut W,
    _shutdown: &AtomicBool,
) -> Result<bool, Error> {
    Ok(true)
}

fn handle_request(tx: &Sender<NetEvent>, peer: SocketAddr, request: Request) -> String {
    let (reply_tx, reply_rx) = channel();
    if tx
        .send(NetEvent {
            peer,
            request,
            reply: reply_tx,
        })
        .is_err()
    {
        eprintln!(
            "handle_request failed trying to open a channel with the state machine for {peer}."
        );
        return "ERR system failed to respond\n".into();
    }
    match reply_rx.recv_timeout(REPLY_TIMEOUT) {
        Ok(Response::Granted) => "GRANTED\n".into(),
        Ok(Response::Denied) => "DENIED\n".into(),
        Err(RecvTimeoutError::Timeout) => {
            eprintln!("System timed out before replying to handle_request.");
            "ERR system timeout\n".into()
        }
        Err(RecvTimeoutError::Disconnected) => {
            eprintln!("System disconnected?!?!?!?");
            "ERR system unavailable\n".into()
        }
    }
}
