use std::fs::File;
use std::io::{Error, Read};
use std::str::from_utf8;
use http4r_core::handler::Handler;
use http4r_core::headers::Headers;
use http4r_core::http_message::{Request, Response};
use http4r_core::http_message::Body::BodyString;
use http4r_core::uri::Uri;

pub struct StaticFileHandler<'a> {
    root: &'a str
}
impl<'a> StaticFileHandler<'a> {
    pub fn new(root: &'a str) -> StaticFileHandler<'a> {
        StaticFileHandler {
            root
        }
    }

    fn ok_or_not_found(file: Result<File, Error>, mut vec: &mut Vec<u8>) -> Response {
        let res = if file.is_err() {
            Response::not_found(Headers::empty(), BodyString("Could not open file."))
        } else {
            let mut file = file.unwrap();
            let metadata = file.metadata();
            if metadata.is_err() {
                Response::not_found(Headers::empty(), BodyString("Could not get metadata for file."))
            } else {
                if !metadata.unwrap().is_file() {
                    Response::not_found(Headers::empty(), BodyString("Not a file but a directory or symlink."))
                } else {
                    let read = file.read_to_end(&mut vec);
                    let str = from_utf8(vec.as_slice());
                    if str.is_err() {
                        Response::not_found(Headers::empty(), BodyString("Could not read body into utf-8."))
                    } else {
                        let body = str.unwrap();
                        Response::ok(Headers::from(vec!(("Content-Type", "text/html"))), BodyString(body))
                    }
                }
            }
        };
        res
    }
}

impl<'a> Handler for StaticFileHandler<'a> {
    fn handle<F>(&mut self, req: Request, fun: F) -> () where F: FnOnce(Response) -> () + Sized {
        match req {
            Request { .. } => {
                let path = if req.uri.path == "/" {
                    self.root.to_owned() + "/index.html"
                } else { req.uri.path.to_string() };
                let file = File::open(path);
                let mut vec = Vec::new();
                let res = Self::ok_or_not_found(file, &mut vec);
                fun(res);
            }
        }
    }
}