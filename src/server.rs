use std::net::{TcpListener, TcpStream};
use std::{thread};
use std::io::{copy, Read, Write};
use crate::httphandler::HttpHandler;
use crate::httpmessage::{bad_request, HttpMessage, length_required, message_from, MessageError, Response};
use crate::httpmessage::Body::{BodyStream, BodyString};
use crate::pool::ThreadPool;

pub struct Server {}

pub struct ServerOptions {
    pub port: Option<u32>,
    pub pool: Option<ThreadPool>,
}

impl Server {
    pub fn new(http_handler: HttpHandler, mut options: ServerOptions) {
        let addr = format!("127.0.0.1:{}", options.port.get_or_insert(7878));
        let listener = TcpListener::bind(addr).unwrap();

        let call_handler = |mut stream: TcpStream, handler: HttpHandler| {
            let buffer = &mut [0 as u8; 16384];
            let first_read = stream.read(buffer).unwrap();
            let result = message_from(buffer, stream.try_clone().unwrap(), first_read);

            Self::write_response(&mut stream, handler, result);

            stream.flush().unwrap();
        };

        match options.pool {
            Some(thread_pool) => {
                for stream in listener.incoming() {
                    thread_pool.execute(move || {
                        call_handler(stream.unwrap(), http_handler)
                    });
                }
            }
            _ => {
                thread::spawn(move || {
                    for stream in listener.incoming() {
                        call_handler(stream.unwrap(), http_handler)
                    }
                });
            }
        }
    }

    fn write_response(mut stream: &mut TcpStream, handler: HttpHandler, result: Result<HttpMessage, MessageError>) {
        match result {
            Err(MessageError::HeadersTooBig(msg)) => {
                let response = bad_request(vec!(), BodyString(msg));
                Self::write_response_to_wire(&mut stream, response)
            },
            Err(MessageError::NoContentLengthOrTransferEncoding(msg)) => {
                let response = length_required(vec!(), BodyString(msg));
                Self::write_response_to_wire(&mut stream, response)
            },
            Ok(HttpMessage::Request(request)) => {
                let response = handler(request);
                Self::write_response_to_wire(&mut stream, response)
            }
            Ok(HttpMessage::Response(response)) => {
                Self::write_response_to_wire(&mut stream, response)
            }
        }
    }

    fn write_response_to_wire(mut stream: &mut TcpStream, mut response: Response) {
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

