use std::io::{copy, Read, Write};
use std::net::TcpStream;
use crate::handler::Handler;
use crate::headers::Headers;
use crate::http_message;
use crate::http_message::{HttpMessage, message_from, Request, Response, with_content_length};
use crate::http_message::Body::{BodyStream, BodyString};

impl Handler for Client {
    fn handle<F>(self: &mut Client, req: Request, fun: F) -> ()
        where F: FnOnce(Response) -> () + Sized {
        let uri = format!("{}:{}", self.base_uri, self.port);
        let mut stream = TcpStream::connect(uri).unwrap();

        let mut _total_length = 0;
        let has_content_length = req.headers.has("Content-Length");
        let has_transfer_encoding = req.headers.has("Transfer-Encoding");

        let headers = Self::ensure_content_length_or_transfer_encoding(&req, has_content_length, has_transfer_encoding)
            .unwrap_or(req.headers);

        let request_string = format!("{} {} HTTP/{}.{}\r\n{}\r\n\r\n",
                                     req.method.value(),
                                     req.uri.to_string(),
                                     req.version.major,
                                     req.version.minor,
                                     headers.to_wire_string());

        stream.write(request_string.as_bytes()).unwrap();

        match (req.body, has_content_length) {
            (BodyStream(ref mut read), true) => {
                let _copy = copy(read, &mut stream).unwrap();
            }
            (BodyStream(ref mut read), false) => {
                let mut vec: Vec<u8> = Vec::new();
                // todo() do read while loop and write as you go a long
                read.read_to_end(&mut vec).unwrap();
                let end = vec!(b'0', b'\r', b'\n', b'\r', b'\n');

                if vec.len() == 0 {
                    let _copy = copy(&mut end.as_slice(), &mut stream).unwrap();
                } else {
                    let mut temp = vec!();
                    temp.push(vec.len() as u8);
                    temp.push(b'\r');temp.push(b'\n');
                    temp.append(&mut vec);
                    temp.push(b'\r');temp.push(b'\n');
                    end.iter().for_each(|b| temp.push(*b));
                    let _copy = copy(&mut temp.as_slice(), &mut stream).unwrap();
                    _total_length += vec.len();
                }
            }
            (BodyString(str), true) => {
                stream.write(str.as_bytes()).unwrap();
            }
            (BodyString(chunk), false) => {
                let mut transfer = "".to_string();
                let length = chunk.len().to_string();
                let end = "0\r\n\r\n";

                transfer.push_str(length.as_str());
                transfer.push_str("\r\n");
                transfer.push_str(chunk);
                transfer.push_str("\r\n");
                transfer.push_str(end);

                stream.write(transfer.as_bytes()).unwrap();
            }
        }

        //todo() read and write timeouts

        let mut buffer = [0; 16384];
        let mut buffer2 = [0; 1048576];
        let first_read = stream.try_clone().unwrap().read(&mut buffer).unwrap();

        let result = message_from(&buffer, stream.try_clone().unwrap(), first_read, &mut buffer2);

        let response = match result {
            Ok(http_message::HttpMessage::Response(res)) => res,
            _ => Response::bad_request(Headers::empty(), BodyString("will happen if server replies with invalid response"))
        };

        fun(response)
    }
}

impl Client {
    fn ensure_content_length_or_transfer_encoding(req: &Request, has_content_length: bool, has_transfer_encoding: bool) -> Option<Headers> {
        if !has_content_length && !has_transfer_encoding && req.body.is_body_string() {
            Some(req.headers.add(("Content-Length", req.body.length().to_string().as_str())))
        } else if !has_content_length && !has_transfer_encoding && req.body.is_body_stream() {
            Some(req.headers.add(("Transfer-Encoding", "chunked")))
        } else {
            None
        }
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
