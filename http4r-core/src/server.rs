use std::net::{TcpListener, TcpStream};
use std::thread;
use std::io::{copy, Read, Write};
use std::sync::Arc;
use crate::handler::Handler;
use crate::headers::Headers;
use crate::http_message::{HttpMessage, message_from, MessageError, Response, with_content_length, write_body, write_chunked_stream, write_chunked_string};
use crate::http_message::Body::{BodyStream, BodyString};
use crate::pool::ThreadPool;

pub struct Server {
    pub port: u16,
}

impl Server where {
    pub fn new(port: u16) -> Server {
        Server {
            port
        }
    }

    pub fn start<F, H>(&mut self, fun: F, mut threadpool_size: Option<u32>)
        where F: Fn() -> Result<H, String> + Send + Sync + 'static, H: Handler {
        let listener = self.listen();
        let handler = Arc::new(fun);

        let x = threadpool_size.get_or_insert(10);
        let pool = ThreadPool::new(x.clone() as usize);

        for stream in listener.incoming() {
            let h = handler.clone();
            pool.execute(move || {
                Self::handle_request(h, stream.unwrap())
            });
        }
    }

    pub fn test<F, H>(&mut self, fun: F)
        where F: Fn() -> Result<H, String> + Send + Sync + 'static, H: Handler {
        let listener = self.listen();
        let handler = Arc::new(fun);

        thread::spawn(move || {
            for stream in listener.incoming() {
                Self::handle_request(handler.clone(), stream.unwrap());
            }
        });
    }

    fn listen(&mut self) -> TcpListener {
        let addr = format!("127.0.0.1:{}", self.port);
        let listener = TcpListener::bind(addr).unwrap();
        self.port = listener.local_addr().unwrap().port();
        listener
    }

    fn handle_request<F, H>(handler: Arc<F>, mut stream: TcpStream)
        where F: Fn() -> Result<H, String> + Send + Sync + 'static, H: Handler {
        let first_read = &mut [0 as u8; 16384];
        let mut chunks_vec = Vec::with_capacity(1048576);
        let first_read_bytes = stream.read(first_read).unwrap();
        let result = message_from(first_read, stream.try_clone().unwrap(), first_read_bytes, &mut chunks_vec);

        match result {
            Err(MessageError::HeadersTooBig(msg)) | Err(MessageError::InvalidContentLength(msg)) => {
                let response = Response::bad_request(Headers::empty(), BodyString(msg.as_str()));
                write_body(&mut stream, HttpMessage::Response(response));
            }
            Err(MessageError::NoContentLengthOrTransferEncoding(msg)) => {
                let response = Response::length_required(Headers::empty(), BodyString(msg.as_str()));
                write_body(&mut stream, HttpMessage::Response(response));
            }
            Ok(HttpMessage::Request(request)) => {
                let mut h = handler().unwrap();
                h.handle(request, |response| {
                    write_body(&mut stream, HttpMessage::Response(response));
                });
            }
            Ok(HttpMessage::Response(response)) => {
                write_body(&mut stream, HttpMessage::Response(response));
            }
        };

        stream.flush().unwrap();
    }

}
