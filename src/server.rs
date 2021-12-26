use std::net::{TcpListener, TcpStream};
use std::{thread};
use std::io::{copy, Read, Write};
use std::sync::Arc;
use crate::handler::Handler;
use crate::http_message::{bad_request, body_string, js_headers_from_string, HttpMessage, length_required, message_from, MessageError, Method, Request, Response, with_content_length, js_headers_to_string};
use crate::http_message::Body::{BodyStream, BodyString};
use crate::pool::ThreadPool;
use wasm_bindgen::prelude::*;
use crate::router::Router;
extern crate console_error_panic_hook;
use std::panic;
use crate::logging_handler::{ConsoleLogger, LoggingHttpHandler, WasmClock};

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
pub struct JSRequest {
    uri: String,
    body: String,
    method: String,
    headers: String,
}

#[allow(non_snake_case)]
#[wasm_bindgen]
pub fn jsRequest(method: &str, uri: &str, body: &str, headers: &str) -> JSRequest {
    let headers = headers.split("; ").fold(vec!(), |mut acc: Vec<(String, String)>, next: &str| {
        let mut split = next.split(": ");
        acc.push((split.next().unwrap().to_string(), split.next().unwrap().to_string()));
        acc
    });
    return JSRequest{
        method: method.to_string(),
        uri: uri.to_string(),
        body: body.to_string(),
        headers: js_headers_to_string(&headers)
    }
}

#[wasm_bindgen]
pub struct JSResponse {
    body: String,
    status: u32,
    headers: String,
}
#[wasm_bindgen]
impl JSResponse {
    pub fn body(&self) -> String {
        (&self.body).to_string()
    }
    pub fn status(&self) -> String {
        (&self.status).to_string()
    }
    pub fn headers(&self) -> String {
        (&self.headers).to_string()
    }
}

#[wasm_bindgen]
pub fn serve(req: JSRequest) -> JSResponse {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    let mut app = RustyApp::new(LoggingHttpHandler::new(ConsoleLogger{}, WasmClock {}, Router {}));
    let request = Request {
        headers: js_headers_from_string(&req.headers),
        method: Method::from(req.method),
        uri: req.uri,
        body: BodyString(req.body),
    };
    let mut response = JSResponse {
        body: "Not found".to_string(),
        headers: "Content-Type: text/plain".to_string(),
        status: 404,
    };
    app.handle(request, |res| {
        response = JSResponse {
            body: body_string(res.body),
            status: res.status.value(),
            headers: js_headers_to_string(&res.headers),
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
