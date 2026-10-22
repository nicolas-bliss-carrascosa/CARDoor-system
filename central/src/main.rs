mod network_ctrl;
mod sql_ctrl;

use std::{sync::mpsc::{Sender, TryRecvError}, thread, time::Duration};

use network_ctrl::tcp_ctrl;

use crate::{network_ctrl::{Request, Response}, sql_ctrl::{Empty, Central}};


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

    let database = Central::<Empty>::new();
    let database = database.connect();
    let mut database = database.prepare_queries();
    
    // A vector for the senders I WILL NOT answer. This is to test the timeout function :)
    let mut stalled: Vec<Sender<Response>> = Vec::new();

    loop {
        loop {
            match net.events.try_recv() {
                Ok(ev) => {
                    match ev.request {
                        Request::Ping(ping_data) => {
                            println!("{} sent a PING from door {}", ev.peer, ping_data.door_id);
                            let _ = ev.reply.send(match database.check_open(ping_data) {
                                true => Response::Granted,
                                false => Response::Denied
                            });
                        }
                        Request::Scan(scan_data) => {
                            println!("{} sent a SCAN from door {} by user {}", ev.peer, scan_data.door_id, scan_data.user_id);
                            match scan_data.user_id.as_str() {
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