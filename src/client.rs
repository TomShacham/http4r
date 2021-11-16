use std::io::{Read, Write};
use std::net::TcpStream;
use std::str;

use crate::httphandler::HttpHandler;
use crate::httpmessage::{Request, Response};
use crate::httpmessage::Status::OK;

impl Client {
    pub fn handle(&self, req: Request) -> Response {
        let uri = format!("{}:{}", self.base_uri, self.port);
        let mut stream = TcpStream::connect(uri).unwrap();
        let request = format!("{} / HTTP/1.1\r\nConnection: close\r\nAccept: text/html\r\n\r\nbody", req.method.value());

        stream.write(request.as_bytes());
        println!("wrote request {}", request.to_string());

        let mut buffer = [0; 1024];
        stream.read(&mut buffer).unwrap();
        println!("read response in client {}", str::from_utf8(&buffer).unwrap());

        let response = Response::from(str::from_utf8(&buffer).unwrap());
        response
    }
}

pub struct Client {
    pub base_uri: String,
    pub port: u32,
}
