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
    fn client_over_http() {
        let port = 7878;
        Server::new(Box::new(Router {}), port);
        let client = Client { base_uri: String::from("127.0.0.1"), port };
        let request = get("".to_string(), vec!(), "".to_string());

        assert_eq!("GET / HTTP/1.1\r\n".to_string(), client.handle(request).body);
    }

    #[test]
    fn router_non_http() {
        let router = Router {};
        let request = get("/".to_string(), vec!(), "".to_string());
        let request_to_no_route = get("no/route/here".to_string(), vec!(), "".to_string());

        assert_eq!(OK, router.handle(request).status);
        assert_eq!(NotFound, router.handle(request_to_no_route).status);
    }

    #[test]
    fn filter() {
        let filter = RawHttpFilter {
            next: Box::new(IdentityFilter {})
        };

        let router = filter.filter(Box::new(Router {}));

    }
}

pub trait HttpHandler {
    fn handle(&self, req: Request) -> Response;
}

pub struct Handler<T>
    where T: Fn(Request) -> Response {
    handler: T,
}

impl<T> HttpHandler for Handler<T>
    where T: Fn(Request) -> Response {
    fn handle(&self, req: Request) -> Response {
        (self.handler)(req)
    }
}


impl<T> Handler<T>
    where T: Fn(Request) -> Response
{
    fn handle(&self, req: Request) -> Response {
        (self.handler)(req)
    }
}

type Header = (String, String);

pub enum HttpMessage {
    Request(Request),
    Response(Response),
}

impl From<Request> for HttpMessage {
    fn from(req: Request) -> Self {
        HttpMessage::Request(req)
    }
}

impl From<Response> for HttpMessage {
    fn from(res: Response) -> Self {
        HttpMessage::Response(res)
    }
}

impl From<HttpMessage> for Request {
    fn from(message: HttpMessage) -> Self {
        match message {
            HttpMessage::Request(req) => req,
            _ => panic!("Not possible")
        }
    }
}

impl From<HttpMessage> for Response {
    fn from(message: HttpMessage) -> Self {
        match message {
            HttpMessage::Response(res) => res,
            _ => panic!("Not possible")
        }
    }
}

pub struct Request {
    pub headers: Vec<Header>,
    pub body: String,
    pub uri: String,
    pub method: Method,
}

pub struct Response {
    pub headers: Vec<Header>,
    pub body: String,
    pub status: Status,
}

pub enum Method {
    GET,
    POST,
    OPTIONS,
    DELETE,
    PATCH,
}

fn ok(headers: Vec<(String, String)>, body: String) -> Response {
    Response { headers, body, status: OK }
}

fn not_found(headers: Vec<(String, String)>, body: String) -> Response {
    Response { headers, body, status: NotFound }
}

fn get(uri: String, headers: Vec<(String, String)>, body: String) -> Request {
    Request { method: GET, headers, body, uri }
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
            "/" => ok(vec!(), "".to_string()),
            _ => not_found(vec!(), "Not found".to_string()),
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
            body: request.to_string(),
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

pub trait Filter {
    fn filter(&self, handler: Box<dyn HttpHandler>) -> Box<dyn HttpHandler>;
}

pub struct RawHttpFilter {
    pub next: Box<dyn Filter>,
}

pub struct IdentityFilter {}

impl Filter for IdentityFilter {
    fn filter(&self, handler: Box<dyn HttpHandler>) -> Box<dyn HttpHandler> {
        Box::new(Handler {
            handler: (move |req| {
                return handler.handle(req);
            })
        })
    }

}

impl Filter for RawHttpFilter {
    fn filter(&self, handler: Box<dyn HttpHandler>) -> Box<dyn HttpHandler> {
        self.next.filter(
            Box::new(Handler {
                handler: (move |request| {
                    let req = add_header(("req-header-1".to_string(), "req-value-1".to_string()), HttpMessage::Request(request)).into();
                    let response = handler.handle(req);
                    add_header(("res-header-1".to_string(), "res-value-1".to_string()), HttpMessage::Response(response)).into()
                })
            })
        )
    }
}

fn add_header(header: Header, to: HttpMessage) -> HttpMessage {
    match to {
        HttpMessage::Request(req) => {
            let mut headers = req.headers.clone();
            headers.push(header);
            HttpMessage::Request(Request {
                headers,
                ..req
            })
        }
        HttpMessage::Response(res) => {
            let mut headers = res.headers.clone();
            headers.push(header);
            HttpMessage::Response(Response {
                headers,
                ..res
            })
        }
    }
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
