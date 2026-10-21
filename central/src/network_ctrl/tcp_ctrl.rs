use std::{error::Error, io::Read, net::{TcpListener, TcpStream}};

fn receive_handshake(server_ip: &str, port: &str) {
    let listener =  match TcpListener::bind(format!("{}:{}", server_ip.to_string(), port.to_string())){
        Ok(a) => a,
        Err(error) => panic!("Failed to initialize the listener on the IP Address: {}:{} with the error {}", server_ip.to_string(), port.to_string(), error)
    };

    for stream in listener.incoming() {
        let mut stream = match stream {
            Ok(a) => a,
            Err(error) => {eprintln!("Error while reading incoming connection. Discarding the attempt. {error}"); continue},
        };

        // SOMETHING SOMETHING AUTHENTICATION
        // assume the computer is authenticated after this (uses the right signature)

        handle_comms(&mut stream);
    }
}

fn handle_comms(mut stream: &mut TcpStream) {
    loop{
        let cmd:&mut [u8; 256]  = &mut [0; 256];

        let sock = match stream.peer_addr() {
            Ok(a) => a,
            Err(error) => { eprintln!("Peer failed to return peer address. Killing the stream. {error}"); return }
        };

        let _ = match stream.read(cmd) {
            Ok(a) => a,
            Err(error) => { eprintln!("Error while reading from socket tied to {}. {error}", sock.ip()); continue }
        };
        let report = match String::from_utf8(cmd.to_vec()) {
            Ok(command) if command.contains("SCAN") => handle_scan(&mut stream, command),
            Ok(command) if command.contains("PING") => handle_ping(&mut stream, command),
            Ok(command) => { eprintln!("Invalid communication from the peer ({command}). Discarding the command"); continue }
            Err(error) => { eprintln!("Error converting the communication from utf8 to string. {error}"); continue }
        };
    }
}

fn handle_scan(stream: &mut TcpStream, command: String) -> String {


    todo!()
}

fn handle_ping(stream: &mut TcpStream, command: String) -> String {
    

    todo!()
}