use std::net::{TcpStream};
use std::io::{Read, Write};
use std::str;
use crate::httphandler::HttpHandler;
use crate::httpmessage::{Request, Response};
use crate::httpmessage::Status::OK;

impl HttpHandler for Client {
    fn handle(&self, req: Request) -> Response {
        let uri = format!("{}:{}", self.base_uri, self.port);
        let mut stream = TcpStream::connect(uri).unwrap();
        let request = format!("{} / HTTP/1.1\r\n", req.method.value());

        stream.write(request.as_bytes());
        println!("wrote request");

        let mut buffer = [0; 1024];
        stream.read(&mut buffer).unwrap();
        println!("read response in client {}", str::from_utf8(&buffer).unwrap());

        Response {
            body: request.to_string(),
            status: OK,
            headers: vec!((String::from("Content-type"), String::from("application/json"))),
        }
    }
}

pub struct Client {
    pub base_uri: String,
    pub port: u32,
}
