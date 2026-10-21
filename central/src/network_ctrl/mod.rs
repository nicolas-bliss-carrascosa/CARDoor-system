use std::{net::SocketAddr, sync::mpsc::Sender};

pub mod tcp_ctrl;

pub enum Request {
    Ping { door_id: String },
    Scan { door_id: String, user_id: String }
}

pub enum Response {
    Granted,
    Denied
}

pub struct NetEvent{
    pub peer: SocketAddr,
    pub request: Request,
    pub reply: Sender<Response>
}
