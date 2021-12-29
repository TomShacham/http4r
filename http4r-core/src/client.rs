use std::io::{copy, Read, Write};
use std::net::TcpStream;
use crate::handler::Handler;
use crate::headers::Headers;
use crate::http_message;
use crate::http_message::{HttpMessage, message_from, Request, Response, with_content_length};
use crate::http_message::Body::{BodyStream, BodyString};

impl Handler for Client {
    fn handle<F>(self: &mut Client, mut req: Request, fun: F) -> ()
        where F: FnOnce(Response) -> () + Sized {
        let uri = format!("{}:{}", self.base_uri, self.port);
        let mut stream = TcpStream::connect(uri).unwrap();
        let request_string = format!("{} / HTTP/1.1\r\n{}\r\n\r\n", req.method.value(), req.headers.to_wire_string());

        stream.write(request_string.as_bytes()).unwrap();
        match req.body {
            BodyStream(ref mut read) => {
                let _copy = copy(read, &mut stream).unwrap();
            }
            BodyString(str) => {
                stream.write(str.as_bytes()).unwrap();
            }
        }

        //todo() read and write timeouts

        let mut buffer = [0; 16384];
        let first_read = stream.try_clone().unwrap().read(&mut buffer).unwrap();

        let result = message_from(&buffer, stream.try_clone().unwrap(), first_read);

        let response = match result {
            Ok(http_message::HttpMessage::Response(res)) => res,
            _ => Response::bad_request(Headers::empty(), BodyString("nah"))
        };

        fun(response)
    }
}


pub struct Client {
    pub base_uri: String,
    pub port: u16,
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
