use std::io::{copy, Error, Read, Write};
use std::net::TcpStream;
use std::{str};
use std::any::Any;
use std::ops::Deref;
use crate::handler::Handler;
use crate::httpmessage::{add_header, bad_request, body_length, header, headers_to_string, Request, Response, response_from, ResponseError};
use crate::httpmessage::Body::{BodyStream, BodyString};

impl Handler for Client {
     fn handle<F>(self: &mut Client, req: Request, mut fun: F) -> ()
        where F: FnOnce(Response) -> () + Sized {
        let uri = format!("{}:{}", self.base_uri, self.port);
        let mut stream = TcpStream::connect(uri).unwrap();
        let mut request = Self::with_content_length(req);
        let request_string = format!("{} / HTTP/1.1\r\n{}\r\n\r\n", request.method.value(), headers_to_string(&request.headers));

        stream.write(request_string.as_bytes());
        match request.body {
            BodyStream(ref mut read) => {
                copy(read, &mut stream);
            },
            BodyString(str) => {
                stream.write(str.as_bytes()).unwrap();
            }
        }

        //todo() FIGURE OUT WHETHER TO CREATE A RESPONSE WITH A BODYSTREAM OR BODYSTRING,
        // DONT JUST READ INTO A BODYSTRING, ACTUALLY BASE IT ON THE CONTENT LENGTH HEADER?
        // TIME TO WRITE THE RESPONSE PARSER, LIKE THE REQUEST ONE THAT GOES THROUGH BYTE BY BYTE.
        // we can look at the content length header if http/1.1 otherwise we wait for EOF or conn close
        // or else it's transfer encoding chunked and we deal with that

        let mut buffer = [0; 16384];
        stream.try_clone().unwrap().read(&mut buffer).unwrap();

        let result = response_from(&buffer, stream.try_clone().unwrap());

        let mut response = match result {
            Ok(res) => res,
            _ => bad_request(vec!(), BodyString("nah".to_string()))
        };

        fun(response)
    }
}

impl Client {
    fn with_content_length(req: Request) -> Request {
        if header(&req.headers, "Content-Length").is_none() {
            return Request {
                headers: add_header(&req.headers, ("Content-Length".to_string(), body_length(&req.body).to_string())),
                ..req
            };
        }
        req
    }
}

pub struct Client {
    pub base_uri: String,
    pub port: u32,
}
