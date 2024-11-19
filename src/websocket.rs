use core::panic;
use std::{net::TcpListener, sync::mpsc::Sender};

pub(crate) fn spawn(sender: Sender<u16>) {
    let listener = TcpListener::bind(format!("127.0.0.1:0"));
    let sent = match &listener {
        Ok(listener) => {
            let port = listener.local_addr().unwrap().port();
            println!("opened port {} for handling websockets", port);
            sender.send(port)
        }
        Err(_) => {
            panic!("could not open port. Thread will now panic!")
        }
    };
    match sent {
        Ok(_) => {
            println!("main thread was notified that the port is now initialized!");
            websocket_main(listener.unwrap());
        }
        Err(e) => {
            println!("could not notify main thread. Terminating thread.");
            panic!("{}", e);
        }
    }
}

fn websocket_main(listener: TcpListener) {
    for stream in listener.incoming() {
        println!("new connection");
        println!("let's listen to what they have to say!");

        let _stream = stream.unwrap(); // Call function to process any incomming connections
    }
}
