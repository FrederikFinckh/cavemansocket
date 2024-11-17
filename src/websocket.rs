use std::{io::Error, net::TcpListener, sync::mpsc::Sender};

pub(crate) fn spawn(request: String, sender: Sender<Result<u16, Error>>) {
    let listener = TcpListener::bind(format!("127.0.0.1:0"));
    let sent = match listener {
        Ok(listener) => {
            let port = listener.local_addr().unwrap().port().clone();
            println!("opened port for handling websockets TODO!{}", port);
            websocket_main(listener);
            sender.send(Ok(port))
        }
        Err(e) => sender.send(Err(e)),
    };
    if let Ok(_) = sent {
        println!("main thread was notified that the port is now initialized!");
    }
}

fn websocket_main(listener: TcpListener) {
    for stream in listener.incoming() {
        println!("new connection");
        println!("let's listen to what they have to say!");
        let _stream = stream.unwrap(); // Call function to process any incomming connections
    }
}
