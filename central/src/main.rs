mod network_ctrl;

use std::{sync::mpsc::{Sender, TryRecvError}, thread, time::Duration};

use network_ctrl::tcp_ctrl;

use crate::network_ctrl::{Request, Response};

/*
     * Okay, so the following are some weird fantasies I have designed to *test* my code... I know, lame
     * Following this demo, one can also figure out how to use the network_ctrl moodule.
     *
     * Run the server first, then run the demo_client.rs using `cargo run --bin demo_client`
     *
     * Yes, demo_client.rs is written by AI. It's 5 am and I am not writing a dumb demo we will not even reuse.
     * Hence, we will NOT submit that module, ever. It just exists as a proof of concept for the communications module.
     *
     * By writing over demo_client.rs, one can add more tests. After some MINOR debugging,
     * currently we pass all the tests it came up with to test this code.
     */

fn main() {
    let net = match tcp_ctrl::start("127.0.0.1:7000") {
        Ok(n) => n,
        Err(e) => { panic!("Network failed to start: {e}") }
    };
    println!("Listenning!!");

    // A vector for the senders I WILL NOT answer. This is to test the timeout function :)
    let mut stalled: Vec<Sender<Response>> = Vec::new();

    loop {
        loop {
            match net.events.try_recv() {
                Ok(ev) => {
                    match ev.request {
                        Request::Ping { door_id: door } => {
                            println!("{} sent a PING from door {door}", ev.peer);
                            let _ = ev.reply.send(Response::Denied); // Now if the database has set the door to open, this would be Response::Granted instead
                        }
                        Request::Scan { door_id: door, user_id: user } => {
                            println!("{} sent a SCAN from door {door} by user {user}", ev.peer);
                            match user.as_str() {
                                "alice" => {let _ = ev.reply.send(Response::Granted); }
                                // We make this little guy wait :(
                                "dna" => stalled.push(ev.reply),
                                _ => {let _ = ev.reply.send(Response::Denied); }
                            }
                        }
                    }
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    eprintln!("Network layer disconnected????");
                    return;
                }
            }

        }
        stalled.retain(|_| true);
        thread::sleep(Duration::from_millis(10));
    }
}