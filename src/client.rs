use std::io::{copy, Read, Write};
use std::net::TcpStream;
use crate::handler::Handler;
use crate::http_message;
use crate::http_message::{bad_request, headers_to_string, HttpMessage, message_from, Request, Response, with_content_length};
use crate::http_message::Body::{BodyStream, BodyString};

impl Handler for Client {
    fn handle<F>(self: &mut Client, req: Request, fun: F) -> ()
        where F: FnOnce(Response) -> () + Sized {
        let uri = format!("{}:{}", self.base_uri, self.port);
        let mut stream = TcpStream::connect(uri).unwrap();
        let mut request = with_content_length(HttpMessage::Request(req)).to_req();
        let request_string = format!("{} / HTTP/1.1\r\n{}\r\n\r\n", request.method.value(), headers_to_string(&request.headers));

        stream.write(request_string.as_bytes()).unwrap();
        match request.body {
            BodyStream(ref mut read) => {
                let _copy = copy(read, &mut stream).unwrap();
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
        let first_read = stream.try_clone().unwrap().read(&mut buffer).unwrap();

        let result = message_from(&buffer, stream.try_clone().unwrap(), first_read);

        let response = match result {
            Ok(http_message::HttpMessage::Response(res)) => res,
            _ => bad_request(vec!(), BodyString("nah".to_string()))
        };

        fun(response)
    }
}


pub struct Client {
    pub base_uri: String,
    pub port: u32,
}
