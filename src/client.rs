use std::io::{copy, Read, Write};
use std::net::TcpStream;
use crate::handler::Handler;
use crate::httpmessage::{add_header, bad_request, body_length, header, headers_to_string, Request, Response, response_from};
use crate::httpmessage::Body::{BodyStream, BodyString};

impl Handler for Client {
    fn handle<F>(self: &mut Client, req: Request, fun: F) -> ()
        where F: FnOnce(Response) -> () + Sized {
        let uri = format!("{}:{}", self.base_uri, self.port);
        let mut stream = TcpStream::connect(uri).unwrap();
        let mut request = Self::with_content_length(req);
        let request_string = format!("{} / HTTP/1.1\r\n{}\r\n\r\n", request.method.value(), headers_to_string(&request.headers));

        stream.write(request_string.as_bytes()).unwrap();
        match request.body {
            BodyStream(ref mut read) => {
                println!("copying request body into tcp stream");
                let copy = copy(read, &mut stream).unwrap();
                println!("copied {} bytes to stream from request", copy);
            }
            BodyString(str) => {
                stream.write(str.as_bytes()).unwrap();
            }
        }

        //todo() FIGURE OUT WHETHER TO CREATE A RESPONSE WITH A BODYSTREAM OR BODYSTRING,
        // DONT JUST READ INTO A BODYSTRING, ACTUALLY BASE IT ON THE CONTENT LENGTH HEADER?
        // TIME TO WRITE THE RESPONSE PARSER, LIKE THE REQUEST ONE THAT GOES THROUGH BYTE BY BYTE.
        // we can look at the content length header if http/1.1 otherwise we wait for EOF or conn close
        // or else it's transfer encoding chunked and we deal with that

        //todo() read and write timeouts

        let mut buffer = [0; 16384];
        println!("reading response into buffer in client");
        let first_read = stream.try_clone().unwrap().read(&mut buffer).unwrap();

        let result = response_from(&buffer, stream.try_clone().unwrap(), first_read);

        let response = match result {
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
