use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::{panic, thread};
use std::str;
use std::time::Duration;
use crate::Method::GET;
use crate::Status::{NotFound, OK};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request() {
        let port = 7878;
        Server::new(Box::new(Router {}), port);
        let client = Client { base_uri: String::from("127.0.0.1"), port };
        let request = get("".to_string(), vec!(), None);

        assert_eq!(Some("GET / HTTP/1.1\r\n".to_string()), client.handle(request).body);
    }

    #[test]
    fn route() {
        let router = Router {};
        let request = get("/".to_string(), vec!(), None);
        let request_to_no_route = get("no/route/here".to_string(), vec!(), None);

        assert_eq!(OK, router.handle(request).status);
        assert_eq!(NotFound, router.handle(request_to_no_route).status);
    }
}

pub trait HttpHandler {
    fn handle(&self, req: Request) -> Response;
}

type Header = (String, String);

pub struct Request {
    pub headers: Vec<Header>,
    pub body: Option<String>,
    pub uri: String,
    pub method: Method,
}

pub enum Method {
    GET,
    POST,
    OPTIONS,
    DELETE,
    PATCH,
}

fn ok(headers: Vec<(String, String)>, body: Option<String>) -> Response {
    Response { headers, body, status: OK }
}

fn not_found(headers: Vec<(String, String)>, body: Option<String>) -> Response {
    Response { headers, body, status: NotFound }
}

fn get(uri: String, headers: Vec<(String, String)>, body: Option<String>) -> Request {
    Request { method: GET, headers, body, uri }
}

pub struct Response {
    pub headers: Vec<Header>,
    pub body: Option<String>,
    pub status: Status,
}

#[derive(PartialEq, Debug)]
pub enum Status {
    OK = 200,
    NotFound = 404,
}

pub struct Client {
    pub base_uri: String,
    pub port: u32,
}

pub struct Router {}

impl HttpHandler for Router {
    fn handle(&self, req: Request) -> Response {
        match req.uri.as_str() {
            "/" => ok(vec!(), None),
            _ => not_found(vec!(), Some(String::from("Not found"))),
        }
    }
}

impl HttpHandler for Client {
    fn handle(&self, req: Request) -> Response {
        let uri = format!("{}:{}", self.base_uri, self.port);
        println!("{}", uri);
        let mut stream = TcpStream::connect(uri).unwrap();
        let request = "GET / HTTP/1.1\r\n";
        stream.write(request.as_bytes());

        println!("wrote request");

        Response {
            body: Some(request.to_string()),
            status: OK,
            headers: vec!((String::from("Content-type"), String::from("application/json"))),
        }
    }
}

fn run_with_server<T>(test: T) -> ()
    where T: FnOnce() -> () + panic::UnwindSafe
{
    let result = panic::catch_unwind(|| {
        test()
    });
    // teardown();
    assert!(result.is_ok())
}

pub struct Server {
    listener: TcpListener,
    handler: Box<dyn HttpHandler>,
    port: u32,
}

impl Server {
    pub fn new(handler: Box<dyn HttpHandler>, port: u32) -> Server {
        let addr = format!("127.0.0.1:{}", port);
        let listener = TcpListener::bind(addr).unwrap();

        fn handle_connection(mut stream: TcpStream) {
            let mut buffer = [0; 1024];

            stream.read(&mut buffer).unwrap();

            println!("{}", str::from_utf8(&buffer).unwrap());

            let response = "HTTP/1.1 200 OK\r\n\r\n";

            stream.write(response.as_bytes()).unwrap();
            stream.flush().unwrap();
        }
        let listener1 = listener.try_clone().unwrap();

        thread::spawn(move || {
            for stream in listener1.incoming() {
                let stream = stream.unwrap();

                handle_connection(stream);
            }
        });

        Server { listener, handler, port }
    }
}

impl HttpHandler for Server {
    fn handle(&self, req: Request) -> Response {
        self.handler.handle(req)
    }
}
