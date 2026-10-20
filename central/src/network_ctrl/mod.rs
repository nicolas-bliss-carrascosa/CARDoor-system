use std::{net::SocketAddr, sync::mpsc::Sender};

pub mod tcp_ctrl;

pub enum Request {
    Ping(PingData),
    Scan(ScanData)
}

pub struct PingData {
    pub door_id: String,
}

pub struct ScanData {
    pub door_id: String,
    pub user_id: String,
}

pub enum Response {
    Granted,
    Denied
}

pub struct NetEvent{
    pub peer: SocketAddr,
    pub request: Request,
    pub reply: Sender<Response>
struct ScanData {
    doorId: usize,
    userId: usize,
}

struct ScanResponse {
    open: bool,
}

struct PingData {
    doorId: usize,
}

struct PingResponse {
    open: bool,
    //TODO: other things we want to be able to update
}