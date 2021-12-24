use std::net::{TcpListener, TcpStream};
use std::{thread};
use std::io::{copy, Read, Write};
use std::sync::Arc;
use crate::handler::Handler;
use crate::http_message::{bad_request, HttpMessage, length_required, message_from, MessageError, Response, with_content_length};
use crate::http_message::Body::{BodyStream, BodyString};
use crate::pool::ThreadPool;

pub struct Server<H> where H: Handler + std::marker::Sync + std::marker::Send + 'static {
    next_handler: H,
}

pub struct ServerOptions {
    pub port: Option<u32>,
    pub pool: Option<ThreadPool>,
}

impl<H> Server<H> where H: Handler + std::marker::Sync + std::marker::Send + 'static {
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

    fn write_response_to_wire(mut stream: &mut TcpStream, mut response: Response) {
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

