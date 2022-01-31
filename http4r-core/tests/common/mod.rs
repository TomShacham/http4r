use std::io::Write;
use std::net::TcpStream;
use http4r_core::handler::Handler;
use http4r_core::headers::Headers;
use http4r_core::http_message;
use http4r_core::http_message::{Body, read_message_from_wire, Request, Response};
use http4r_core::http_message::Body::BodyString;

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

pub struct MalformedChunkedEncodingClient {
    pub port: u16,
}

impl Handler for MalformedChunkedEncodingClient {
    fn handle<F>(self: &mut MalformedChunkedEncodingClient, _req: Request, fun: F) -> ()
        where F: FnOnce(Response) -> () + Sized {
        let uri = format!("127.0.0.1:{}", self.port);
        let mut stream = TcpStream::connect(uri).unwrap();

        stream.write("GET / HTTP/1.1\r\nTransfer-Encoding: Chunked\r\n\r\n5\r\nhello\r\nX".as_bytes()).unwrap();

        let mut reader: &mut [u8] = &mut [0; 4096];
        let mut chunks_vec = Vec::with_capacity(1048576);
        let mut start_line_writer = Vec::with_capacity(16384);
        let mut headers_writer = Vec::with_capacity(16384);
        let mut trailers_writer = Vec::with_capacity(16384);
        let result = read_message_from_wire(stream.try_clone().unwrap(), &mut reader, &mut start_line_writer, &mut headers_writer, &mut chunks_vec, &mut trailers_writer, None);

        let response = match result {
            Ok(http_message::HttpMessage::Response(res)) => res,
            _ => Response::bad_request(Headers::empty(), BodyString("will happen if server replies with invalid response"))
        };

        fun(response)
    }
}
