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
            err: "".to_string()
        }
    }
}

impl Handler for Client {
    fn handle<F>(self: &mut Client, req: Request, fun: F) -> ()
        where F: FnOnce(Response) -> () + Sized {
        let uri = format!("{}:{}", self.base_uri, self.port);
        let mut stream = TcpStream::connect(uri).unwrap();

        write_body(&mut stream, HttpMessage::Request(req));

        let mut reader: &mut [u8] = &mut [0; 4096];
        let mut chunks_vec = Vec::with_capacity(1048576);
        let mut start_line_writer = Vec::with_capacity(16384);
        let mut headers_writer = Vec::with_capacity(16384);
        let mut trailers_writer = Vec::with_capacity(16384);
        let result = message_from(stream.try_clone().unwrap(), &mut reader, &mut start_line_writer, &mut headers_writer, &mut trailers_writer, &mut chunks_vec);

        let response = match result {
            Ok(http_message::HttpMessage::Response(res)) => res,
            Err(e) => {
                self.err = e.to_string();
                Response::bad_request(Headers::empty(), BodyString(self.err.as_str()))
            },
            _ => Response::bad_request(Headers::empty(), BodyString("will happen if server replies with invalid response"))
        };

        fun(response)
    }
}

pub struct Client {
    pub base_uri: String,
    pub port: u16,
    options: ClientOptions,
    pub err: String,
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
