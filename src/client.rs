use std::io::{Read, Write};
use std::net::TcpStream;
use std::str;
use crate::httpmessage::{add_header, body_length, header, headers_to_string, Request, Response};

impl Client {
    pub fn handle(&self, req: Request) -> Response {
        let uri = format!("{}:{}", self.base_uri, self.port);
        let mut stream = TcpStream::connect(uri).unwrap();

        let request = Self::with_content_length(req);
        let request_string = format!("{} / HTTP/1.1\r\n{}\r\n\r\nrequest body", request.method.value(), headers_to_string(&request.headers));

        stream.write(request_string.as_bytes()).unwrap();

        //todo() FIGURE OUT WHETHER TO RESPOND WITH A BODYSTREAM OR BODYSTRING,
        // DONT JUST READ INTO A BODYSTRING, ACTUALLY BASE IT ON THE CONTENT LENGTH HEADER?
        // TIME TO WRITE THE RESPONSE PARSER, LIKE THE REQUEST ONE THAT GOES THROUGH BYTE BY BYTE.
        let mut buffer = [0; 4096];
        stream.try_clone().unwrap().read(&mut buffer).unwrap();

        let str1 = str::from_utf8(&buffer).unwrap();
        Response::from(str1.to_string())
    }

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
