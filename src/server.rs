use std::net::{TcpListener, TcpStream};
use std::{thread};
use std::io::{copy, Read, Write};
use std::sync::Arc;
use crate::handler::Handler;
use crate::http_message::{bad_request, body_string, HttpMessage, length_required, message_from, MessageError, Method, Request, Response, with_content_length};
use crate::http_message::Body::{BodyStream, BodyString};
use crate::pool::ThreadPool;
use wasm_bindgen::prelude::*;
use crate::logging_handler::LoggingHttpHandler;
use crate::router::Router;

pub struct Server<H> where H: Handler + Sync + Send + 'static {
    _next_handler: H,
}

pub struct ServerOptions {
    pub port: Option<u32>,
    pub pool: Option<ThreadPool>,
}

impl<H> Server<H> where H: Handler + Sync + Send + 'static {
    pub fn new<F>(fun: F, mut options: ServerOptions)
        where F: Fn() -> Result<H, String> + Send + Sync + 'static {
        let addr = format!("127.0.0.1:{}", options.port.get_or_insert(7878));
        let listener = TcpListener::bind(addr).unwrap();
        let handler = Arc::new(fun);

        thread::spawn(move || {
            for stream in listener.incoming() {
                let mut arc = handler().unwrap();
                let mut stream = stream.unwrap();
                let buffer = &mut [0 as u8; 16384];
                let first_read = stream.read(buffer).unwrap();
                let result = message_from(buffer, stream.try_clone().unwrap(), first_read);

                match result {
                    Err(MessageError::HeadersTooBig(msg)) => {
                        let response = bad_request(vec!(), BodyString(msg));
                        Self::write_response_to_wire(&mut stream, response)
                    }
                    Err(MessageError::NoContentLengthOrTransferEncoding(msg)) => {
                        let response = length_required(vec!(), BodyString(msg));
                        Self::write_response_to_wire(&mut stream, response)
                    }
                    Ok(HttpMessage::Request(request)) => {
                        arc.handle(request, |res| {
                            Self::write_response_to_wire(&mut stream, res)
                        });
                    }
                    Ok(HttpMessage::Response(response)) => {
                        Self::write_response_to_wire(&mut stream, response)
                    }
                };

                stream.flush().unwrap();
            }
        });
    }

    fn write_response_to_wire(mut stream: &mut TcpStream, response: Response) {
        let mut response = with_content_length(HttpMessage::Response(response)).to_res();
        let mut returning: String = response.resource_and_headers();

        match response.body {
            BodyString(body_string) => {
                returning.push_str(&body_string);
                stream.write(returning.as_bytes()).unwrap();
            }
            BodyStream(ref mut body_stream) => {
                let _status_line_and_headers = stream.write(returning.as_bytes()).unwrap();
                let _copy = copy(body_stream, &mut stream).unwrap();
            }
        }
    }
}

#[wasm_bindgen]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JSRequest {
    uri: String,
    body: String,
    method: String,
    headers: Vec<(String, String)>,
}

#[wasm_bindgen]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JSResponse {
    body: String,
    status: String,
    headers: Vec<(String, String)>,
}

#[wasm_bindgen]
pub fn serve(req: JSRequest) -> JSResponse {
    let mut app = RustyApp::new(LoggingHttpHandler::new(Router {}));
    let request = Request {
        headers: req.headers,
        method: Method::from(req.method),
        uri: req.uri,
        body: BodyString(req.body),
    };
    let mut response = JSResponse {
        body: "Not found".to_string(),
        headers: vec!(),
        status: "404".to_string(),
    };
    app.handle(request, |res| {
        response = JSResponse {
            body: body_string(res.body),
            status: res.status.to_string(),
            headers: res.headers,
        }
    });
    response
}

pub struct RustyApp<H> where H: Handler {
    next_handler: H,
}

impl<H> RustyApp<H> where H: Handler {
    pub fn new(next_handler: H) -> RustyApp<H> {
        RustyApp {
            next_handler
        }
    }
}

impl<H> Handler for RustyApp<H> where H: Handler {
    fn handle<F>(&mut self, req: Request, fun: F) -> () where F: FnOnce(Response) -> () + Sized {
        println!("App start");
        self.next_handler.handle(req, |res| {
            fun(res);
            println!("App end")
        })
    }
}
