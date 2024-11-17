use std::{
    fs::{read_to_string, File},
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::mpsc,
    thread,
};

mod websocket;

const PUBLIC_RESOURCES: [&str; 3] = [
    "./frontend/index.html",
    "./frontend/script.js",
    "./frontend/favicon.ico",
];

enum Resource {
    Index,
    Script,
    Favicon,
}

const HTTP_OK: &str = "HTTP/1.1 200 OK\r\n";

fn create_tcp_listener(port: u16) -> Result<TcpListener, std::io::Error> {
    println!("trying to run webserver on port {port}");
    TcpListener::bind(format!("127.0.0.1:{}", port))
}

fn main() {
    for stream in create_tcp_listener(6969).unwrap().incoming() {
        println!("new connection");
        println!("let's listen to what they have to say!");
        let _stream = stream.unwrap(); // Call function to process any incomming connections
        handle_connection(_stream);
    }
    println!("Hello, world!");
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();
    let request = String::from_utf8_lossy(&buffer[..]);
    println!("---------------------------------");
    println!("Request: {}", request);

    if request.starts_with("GET / HTTP/1.1") {
        handle(stream, Resource::Index);
    } else if request.starts_with("GET /script.js HTTP/1.1") {
        println!("someone wants the JS!");
        handle(stream, Resource::Script);
    } else if request.starts_with("GET /favicon.ico HTTP/1.1") {
        handle(stream, Resource::Favicon);
    } else if request.starts_with("POST /host HTTP/1.1") {
        handle_host(stream, request.to_string());
    }
}

fn handle_host(stream: TcpStream, request: String) {
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        println!("it seems like someone wants to host a new game. I have just spawned a new thread to handle this!");
        println!("let's bind to a fresh port (given by the OS) and listen to connections on that port. We return the portnumber and keep track of the host of the game. They should be in the request.");
        println!("Once the port is opened we will wait for incoming websocket connections and broadcast all the messages to everyone!");
        println!("to communicate we use a channel that was created in the main thread and this new thread now has the sender. It will send a message with the portnumber as soon as everything is set up");
        websocket::spawn(request, sender);
    });
    match receiver.recv() {
        Ok(port) => println!("received {:?}", port),
        Err(e) => println!("something went wrong receiving the opened port: {}", e),
    }
    serve_html(stream, "./frontend/index.html"); //for now
}

fn handle(stream: TcpStream, resource: Resource) {
    match resource {
        Resource::Index => {
            serve_html(stream, "./frontend/index.html");
        }
        Resource::Script => {
            serve_script(stream, "./frontend/script.js");
        }
        Resource::Favicon => {
            serve_image(stream, "./frontend/favicon.ico");
        }
    }
}

fn serve_script(stream: TcpStream, path: &str) {
    let bytes = if !PUBLIC_RESOURCES.contains(&path) {
        forbidden()
    } else {
        let response = read_to_string(path).unwrap().to_string();
        let response_http = format!(
            "{}Content-Length: {}\r\nContent-Type: text/javascript\r\n\r\n{}",
            HTTP_OK,
            response.len(),
            response
        );
        response_http.as_bytes().to_owned()
    };
    serve_bytes(stream, bytes);
}
fn serve_html(stream: TcpStream, path: &str) {
    let bytes = if !PUBLIC_RESOURCES.contains(&path) {
        forbidden()
    } else {
        let response = read_to_string(path).unwrap().to_string();
        let response_http = format!(
            "{}Content-Length: {}\r\nContent-Type: text/html\r\n\r\n{}",
            HTTP_OK,
            response.len(),
            response
        );
        response_http.as_bytes().to_owned()
    };
    serve_bytes(stream, bytes);
}

fn serve_image(stream: TcpStream, path: &str) {
    let bytes = if !PUBLIC_RESOURCES.contains(&path) {
        forbidden()
    } else {
        let mut f = File::open(path).unwrap();
        let mut buffer = Vec::new();
        f.read_to_end(&mut buffer).unwrap();
        let mut response = format!(
            "{}Content-Length: {}\r\nContent-Type: image/png\r\n\r\n",
            HTTP_OK,
            buffer.len()
        )
        .as_bytes()
        .to_vec();
        response.append(&mut buffer);
        response
    };
    serve_bytes(stream, bytes);
}

fn serve_bytes(mut stream: TcpStream, bytes: Vec<u8>) {
    match stream.write(&bytes) {
        Ok(_) => {
            println!("successfully served response!");
            stream.flush().unwrap();
        }
        Err(e) => {
            println!("there was an error sending response to client! {:?}", e);
        }
    }
    println!("-------------------------------");
}

fn forbidden() -> Vec<u8> {
    "HTTP/1.1 403 Forbidden".as_bytes().to_vec()
}
