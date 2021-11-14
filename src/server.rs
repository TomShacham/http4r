use std::net::{TcpListener, TcpStream};
use std::{str, thread};
use std::borrow::Borrow;
use std::io::{Read, Write};
use std::sync::{Arc, mpsc};
use crate::httphandler::HttpHandler;
use crate::httpmessage::{Method, Request, Response};
use crate::httpmessage::Method::GET;

pub struct Server {
    listener: TcpListener,
    handler: Box<dyn HttpHandler>,
    port: u32,
}

impl Server {
    pub fn new(handler: Box<dyn HttpHandler>, port: u32) {
        let addr = format!("127.0.0.1:{}", port);
        let listener = TcpListener::bind(addr).unwrap();
        // let (tx, rx) = mpsc::channel();

        let handle_connection = |mut stream: TcpStream| {
            let mut buffer = [0; 1024];

            stream.read(&mut buffer).unwrap();

            let string = str::from_utf8(&buffer).unwrap();
            println!("got request in server {}", &string);

            let string = string.split(" ").take(2).collect::<Vec<&str>>();

            let request = Request {
                method: Method::from(string[0].to_string()),
                headers: vec!(),
                body: "".to_string(),
                uri: string[1].to_string(),
            };

            // let response = handler.handle(request);

            // println!("response body is {}", response.body);

            stream.write(string.join("").as_bytes()).unwrap();
            stream.flush().unwrap();
        };

        thread::spawn(move || {
            for stream in listener.incoming() {
                let stream = stream.unwrap();
                handle_connection(stream);
            }
        });

    }
}

impl HttpHandler for Server {
    fn handle(&self, req: Request) -> Response {
        self.handler.handle(req)
    }
}
