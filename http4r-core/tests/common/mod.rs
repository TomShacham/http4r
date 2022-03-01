use std::collections::HashMap;
use std::io::Write;
use std::net::TcpStream;
use std::time::Instant;

use http4r_core::handler::Handler;
use http4r_core::headers::Headers;
use http4r_core::http_message;
use http4r_core::http_message::{Body, read_message_from_wire, Request, Response};
use http4r_core::http_message::Body::BodyString;

use crate::http_message::{Body, Request, Response};

pub struct Router {}

impl Handler for Router {
    fn handle<F>(&mut self, req: Request, fun: F) -> () where F: FnOnce(Response) -> () + Sized {
        let response = match req.uri.to_string().as_str() {
            "/" => Response::ok(Headers::empty(), Body::empty()),
            _ => Response::not_found(Headers::empty(), BodyString("Not found")),
        };
        fun(response)
    }
}

pub struct PassThroughHandler {}

impl Handler for PassThroughHandler {
    fn handle<F>(&mut self, req: Request, fun: F) -> () where F: FnOnce(Response) -> () + Sized {
        let response = Response::ok(req.headers, req.body).with_trailers(req.trailers);
        fun(response);
    }
}

pub struct EchoBodyHandler {}

impl Handler for EchoBodyHandler {
    fn handle<F>(&mut self, req: Request, fun: F) -> () where F: FnOnce(Response) -> () + Sized {
        let response = Response::ok(Headers::empty(), req.body);
        fun(response);
    }
}


pub struct SetContentEncodingToNoneAndEchoHeaders {}

impl Handler for SetContentEncodingToNoneAndEchoHeaders {
    fn handle<F>(&mut self, _req: Request, fun: F) -> () where F: FnOnce(Response) -> () + Sized {
        let response = Response::ok(Headers::from(vec!(("Content-Encoding", "none"))), Body::empty());
        fun(response);
    }
}



pub struct PassHeadersAsBody {}

impl Handler for PassHeadersAsBody {
    fn handle<F>(&mut self, req: Request, fun: F) -> () where F: FnOnce(Response) -> () + Sized {
        // this will not get changed by the write_response_to_wire logic that strips various headers etc
        // because we are setting the body here in the handler, before it gets written back to the wire
        let string = req.headers.to_wire_string();
        let req_headers_string = string.as_str();
        let response = Response::ok(
            Headers::from(vec!(("Content-Length", string.len().to_string().as_str()))),
            BodyString(req_headers_string));
        fun(response);
    }
}

pub struct MalformedChunkedEncodingClient {
    pub port: u16,
}

impl Handler for MalformedChunkedEncodingClient {
    fn handle<F>(self: &mut MalformedChunkedEncodingClient, _req: Request, fun: F) -> ()
        where F: FnOnce(Response) -> () + Sized {
        let uri = format!("127.0.0.1:{}", self.port);
        let mut stream = TcpStream::connect(uri).unwrap();

        stream.write("GET / HTTP/1.1\r\nTransfer-Encoding: chunked\r\n\r\n5\r\nhello\r\nX\r\n\r\n".as_bytes()).unwrap();

        let mut reader: &mut [u8] = &mut [0; 4096];
        let mut chunks_writer = Vec::with_capacity(1048576);
        let mut compress_writer = Vec::with_capacity(1048576);
        let mut start_line_writer = Vec::with_capacity(16384);
        let mut headers_writer = Vec::with_capacity(16384);
        let mut trailers_writer = Vec::with_capacity(16384);
        let result = read_message_from_wire(stream.try_clone().unwrap(), &mut reader, &mut start_line_writer, &mut headers_writer, &mut chunks_writer, &mut compress_writer, &mut trailers_writer);

        let response = match result {
            Ok(http_message::HttpMessage::Response(res)) => res,
            _ => Response::bad_request(Headers::empty(), BodyString("will happen if server replies with invalid response"))
        };

        fun(response)
    }
}

pub struct LoggingHttpHandler<H, C, L> where H: Handler, C: Clock, L: Logger {
    pub log: Vec<String>,
    pub next_handler: H,
    pub clock: C,
    pub logger: L,
}

pub trait Clock {
    fn now(&mut self) -> Instant;
}

pub trait Logger {
    fn log(&mut self, line: &str);
}


pub struct RustLogger {}

impl Logger for RustLogger {
    fn log(&mut self, line: &str) {
        println!("{}", line)
    }
}

pub struct WasmClock {}

impl Clock for WasmClock {
    fn now(&mut self) -> Instant {
        Instant::now()
    }
}

impl<H, C, L> LoggingHttpHandler<H, C, L> where H: Handler, C: Clock, L: Logger {
    pub fn new(logger: L, clock: C, next: H) -> LoggingHttpHandler<H, C, L> {
        LoggingHttpHandler {
            log: vec!(),
            next_handler: next,
            clock,
            logger,
        }
    }
}

impl<H, C, L> Handler for LoggingHttpHandler<H, C, L> where H: Handler, C: Clock, L: Logger {
    fn handle<F>(self: &mut LoggingHttpHandler<H, C, L>, req: Request, fun: F) -> ()
        where F: FnOnce(Response) -> () + Sized {
        let start = self.clock.now();
        let req_string = format!("{} to {}", req.method.value().to_string(), req.uri);
        self.next_handler.handle(req, |res| {
            let status = res.status.to_string();
            fun(res);
            self.log.push(format!("{} => {} took {} μs", req_string, status, start.elapsed().as_micros()));
            self.logger.log(format!("{}", self.log.join("\n")).as_str());
        });
    }
}

type Env = HashMap<String, String>;

pub struct RedirectToHttpsHandler<H> where H: Handler {
    next_handler: H,
    env: Env,
}

impl<H> Handler for RedirectToHttpsHandler<H> where H: Handler {
    fn handle<'a, F>(self: &mut RedirectToHttpsHandler<H>, req: Request<'a>, fun: F) -> ()
        where F: FnOnce(Response) -> () + Sized {
        if self.env.get("environment") == Some(&"production".to_string()) && req.uri.scheme != Some("https") {
            let redirect = Response::moved_permanently(
                Headers::from(vec!(("Location", req.uri.with_scheme("https").to_string().as_str()))),
                Body::empty());
            return fun(redirect);
        }
        self.next_handler.handle(req, fun);
    }
}

impl<H> RedirectToHttpsHandler<H> where H: Handler {
    pub fn new(handler: H, env: Env) -> RedirectToHttpsHandler<H> {
        RedirectToHttpsHandler {
            next_handler: handler,
            env,
        }
    }
}
