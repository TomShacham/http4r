use std::io::{Read};
use std::net::TcpStream;

use crate::handler::Handler;
use crate::headers::Headers;
use crate::http_message;
use crate::http_message::{HttpMessage, message_from, Request, Response, with_content_length, write_body};
use crate::http_message::Body::{BodyString};

impl Client {
    pub fn new(base_uri: &str, port: u16, options: Option<ClientOptions>) -> Client {
        Client {
            base_uri: base_uri.to_string(),
            port,
            options: options.unwrap_or(ClientOptions {
                headers_size: 16384,
                trailers_size: 16384,
            }),
        }
    }
}

impl Handler for Client {
    fn handle<F>(self: &mut Client, req: Request, fun: F) -> ()
        where F: FnOnce(Response) -> () + Sized {
        let uri = format!("{}:{}", self.base_uri, self.port);
        let mut stream = TcpStream::connect(uri).unwrap();

        write_body(&mut stream, HttpMessage::Request(req));

        let mut chunks_vec = Vec::with_capacity(1048576);
        let mut first_read = [0; 16384];
        let first_read_bytes = stream.try_clone().unwrap().read(&mut first_read).unwrap();

        let result = message_from(&mut first_read, stream.try_clone().unwrap(), first_read_bytes, &mut chunks_vec, self.options.headers_size, self.options.trailers_size);

        let response = match result {
            Ok(http_message::HttpMessage::Response(res)) => res,
            Err(e) => Response::bad_request(Headers::empty(), BodyString("An error occurred")),
            _ => Response::bad_request(Headers::empty(), BodyString("will happen if server replies with invalid response"))
        };

        fun(response)
    }
}

pub struct Client {
    pub base_uri: String,
    pub port: u16,
    options: ClientOptions,
}

pub struct ClientOptions {
    headers_size: usize,
    trailers_size: usize,
}

pub struct WithContentLength<H> where H: Handler {
    next_handler: H,
}

impl<H> WithContentLength<H> where H: Handler {
    pub fn new(next_handler: H) -> WithContentLength<H> {
        WithContentLength {
            next_handler
        }
    }
}

impl<H> Handler for WithContentLength<H> where H: Handler {
    fn handle<F>(self: &mut WithContentLength<H>, req: Request, fun: F) -> ()
        where F: FnOnce(Response) -> () + Sized {
        let request = with_content_length(HttpMessage::Request(req)).to_req();
        self.next_handler.handle(request, |res| {
            fun(res)
        })
    }
}
