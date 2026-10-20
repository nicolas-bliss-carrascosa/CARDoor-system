pub mod tcp_int;
mod tcp_ctrl;

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