use std::net::{TcpListener, TcpStream};
use std::{str, thread};
use std::borrow::Borrow;
use std::io::{copy, Read, Write};
use std::ops::Deref;
use std::sync::{Arc, mpsc, Mutex};
use std::sync::mpsc::Receiver;
use crate::headers::add_header;
use crate::httphandler::HttpHandler;
use crate::httpmessage::{bad_request, Body, content_length_header, get, header, HttpMessage, length_required, ok, Request, request_from, RequestError, Response};
use crate::httpmessage::Body::{BodyStream, BodyString};
use crate::pool::Message::NewJob;
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
            let mut buffer = &mut [0 as u8; 16384];
            stream.read(buffer).unwrap();
            let result = request_from(buffer, stream.try_clone().unwrap());

            Self::write_response(&mut stream, handler, result);
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

    fn write_response(mut stream: &mut TcpStream, handler: HttpHandler, result: Result<Request, RequestError>) {
        match result {
            Err(RequestError::HeadersTooBig(msg)) => {
                let mut response = bad_request(vec!(), BodyString(msg));
                Self::write_response_to_wire(&mut stream, response)
            },
            Err(RequestError::NoContentLengthOrTransferEncoding(msg)) => {
                let mut response = length_required(vec!(), BodyString(msg));
                Self::write_response_to_wire(&mut stream, response)
            },
            Ok(request) => {
                let mut response = handler(request);
                Self::write_response_to_wire(&mut stream, response)
            }
        }
        stream.flush().unwrap();
    }

    fn write_response_to_wire(mut stream: &mut TcpStream, mut response: Response) {
        let mut returning: String = response.resource_and_headers();

        match response.body {
            BodyString(body_string) => {
                returning.push_str(&body_string);
                &stream.write(returning.as_bytes());
            }
            BodyStream(ref mut body_stream) => {
                &stream.write(returning.as_bytes());
                copy(body_stream, &mut stream);
            }
        }
    }
}

