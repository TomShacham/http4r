use std::net::{TcpListener, TcpStream};
use std::thread;
use std::io::{Write};
use std::sync::Arc;
use crate::handler::Handler;
use crate::headers::Headers;
use crate::http_message::{HttpMessage, message_from, MessageError, Response, write_body};
use crate::http_message::Body::{BodyString};
use crate::pool::ThreadPool;

pub struct Server {
    pub port: u16,
    // options: ServerOptions,
}

pub struct ServerOptions {
    pub threadpool_size: usize,
    pub headers_size: usize,
    pub trailers_size: usize,
}

impl Server where {
    pub fn new(port: u16) -> Server {
        Server {
            port,
            // options: options.unwrap_or(ServerOptions {
            //     headers_size: 16384,
            //     trailers_size: 16384,
            //     threadpool_size: 10,
            // })
        }
    }

    pub fn start<F, H>(&mut self, fun: F)
        where F: Fn() -> Result<H, String> + Send + Sync + 'static, H: Handler {
        let listener = self.listen();
        let handler = Arc::new(fun);

        let pool = ThreadPool::new(10 as usize);

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
        let mut reader: &mut [u8] = &mut [0; 4096];
        let mut chunks_vec = Vec::with_capacity(1048576);
        let mut start_line_writer = Vec::with_capacity(16384);
        let mut headers_writer = Vec::with_capacity(16384);
        let mut trailers_writer = Vec::with_capacity(16384);

        let result = message_from(
            stream.try_clone().unwrap(),
            &mut reader,
            &mut chunks_vec,
            &mut start_line_writer,
            &mut headers_writer,
            &mut trailers_writer,
        );

        match result {
            Err(MessageError::HeadersTooBig(msg))
            | Err(MessageError::TrailersTooBig(msg))
            | Err(MessageError::InvalidContentLength(msg))
            | Err(MessageError::StartLineTooBig(msg))
            => {
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
