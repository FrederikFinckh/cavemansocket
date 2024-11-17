use std::{
    fs::{read_to_string, File},
    io::{Read, Write},
    net::{TcpListener, TcpStream},
};

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
    println!("Request: {}", request);

    if request.starts_with("GET / HTTP/1.1") {
        handle(stream, Resource::Index);
    } else if request.starts_with("GET /script.js HTTP/1.1") {
        println!("someone wants the JS!");
        handle(stream, Resource::Script);
    } else if request.starts_with("GET /favicon.ico HTTP/1.1") {
        handle(stream, Resource::Favicon);
    }
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
            "HTTP/1.1 200 OK \r\nContent-Length: {}\r\nContent-Type: text/javascript\r\n\r\n{}",
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
            "HTTP/1.1 200 OK \r\nContent-Length: {}\r\nContent-Type: text/html\r\n\r\n{}",
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
        buffer
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
}

fn forbidden() -> Vec<u8> {
    "HTTP/1.1 403 Forbidden".as_bytes().to_vec()
}
