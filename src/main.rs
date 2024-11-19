use std::sync::{RwLock, RwLockWriteGuard};
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

const HTTP_OK: &str = "HTTP/1.1 200 OK\r\n";

fn create_tcp_listener(port: u16) -> Result<TcpListener, std::io::Error> {
    println!("trying to run webserver on port {port}");
    TcpListener::bind(format!("127.0.0.1:{}", port))
}

#[derive(Clone)]
struct Session {
    port: u16,
    hosting_user: User,
    joined_users: Vec<User>,
}

impl Session {
    fn to_json(&self) -> String {
        let joined_users_strings: Vec<String> = self
            .joined_users
            .iter()
            .map(|user| format!("\"{}\"", user.name.clone()))
            .collect();
        format!(
            "{{\"port\": {}, \"hosting_user\": \"{}\", \"joined_users\": [{}]}}",
            self.port,
            self.hosting_user.name,
            joined_users_strings.join(",")
        )
    }
}

#[derive(Clone)]
struct User {
    name: String,
}

fn main() {
    let sessions: RwLock<Vec<Session>> = RwLock::new(vec![]);
    for stream in create_tcp_listener(6969).unwrap().incoming() {
        println!("new connection");
        println!("let's listen to what they have to say!");
        handle_connection(stream.unwrap(), sessions.write().unwrap());
    }
}

fn handle_connection(mut stream: TcpStream, sessions: RwLockWriteGuard<'_, Vec<Session>>) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();
    let request = String::from_utf8_lossy(&buffer[..]);
    println!("---------------------------------");
    println!("Request: {}", request);

    if request.starts_with("GET / HTTP/1.1") {
        serve_html(stream, "./frontend/index.html");
    } else if request.starts_with("GET /script.js HTTP/1.1") {
        println!("someone wants the JS!");
        serve_script(stream, "./frontend/script.js");
    } else if request.starts_with("GET /favicon.ico HTTP/1.1") {
        serve_image(stream, "./frontend/favicon.ico");
    } else if request.starts_with("GET /sessions HTTP/1.1") {
        let sessions_strings: Vec<String> = sessions
            .to_vec()
            .iter()
            .map(|session| format!("{}", session.to_json()))
            .collect();
        serve_json(stream, format!("[{}]", sessions_strings.join(",")));
    } else if request.starts_with("POST /host HTTP/1.1") {
        handle_host(stream, sessions, request.to_string());
    }
}

fn handle_host(
    stream: TcpStream,
    mut sessions: RwLockWriteGuard<'_, Vec<Session>>,
    request: String,
) {
    let username_pattern = "{\"username\":\"";
    let username = match request.find(username_pattern) {
        Some(hosting_user_index) => &request[hosting_user_index + username_pattern.len()..]
            .to_string()
            .split_once("\"}")
            .unwrap()
            .0
            .to_string(),
        None => return,
    };
    println!(
        "it seems like {} wants to host a new game. I will spawn a new thread to handle this!",
        username
    );
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        println!("let's bind to a fresh port (given by the OS) and listen to connections on that port. We return the portnumber and keep track of the host of the game. They should be in the request.");
        println!("Once the port is opened we will wait for incoming websocket connections and broadcast all the messages to everyone connected to it!");
        println!("to communicate we use a channel that was created in the main thread and this new thread now has the sender. It will send a message with the portnumber as soon as everything is set up");
        websocket::spawn(sender);
    });
    println!("waiting for other thread to send a response");
    match receiver.recv() {
        Ok(port) => {
            println!("received {:?}", port);
            sessions.push(Session {
                port,
                hosting_user: User {
                    name: username.to_string(),
                },
                joined_users: vec![],
            });
            serve_json(
                stream,
                format!("{{\"port\": {}, \"username\": \"{}\"}}", port, username),
            );
        }
        Err(e) => {
            println!("something went wrong receiving the opened port: {}", e);
        }
    }
}

fn serve_json(stream: TcpStream, json_body: String) {
    println!("serving json: \n{}", json_body);
    let response = format!(
        "{}Content-length: {}\r\nContent-Type: text/json\r\n\r\n{}",
        HTTP_OK,
        json_body.len(),
        json_body
    );
    serve_bytes(stream, response.as_bytes().to_vec());
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
