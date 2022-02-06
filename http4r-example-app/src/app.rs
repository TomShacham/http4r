use std::fs;
use std::fs::File;
use std::io::Read;
use std::str::from_utf8;
use http4r_core::handler::Handler;
use http4r_core::headers::Headers;
use http4r_core::http_message::{Request, Response};
use http4r_core::http_message::Body::BodyString;
use http4r_core::uri::Uri;

pub struct App {}

impl App {
    pub fn new() -> App {
        App {}
    }
}

impl Handler for App {
    fn handle<F>(&mut self, req: Request, fun: F) -> () where F: FnOnce(Response) -> () + Sized {
        match req {
            Request { uri: Uri { path: "/", .. }, .. } => {
                let file = File::open("./resources/html/index.html");
                let mut vec = Vec::new();
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
                fun(res);
            }
            _ => fun(Response::not_found(Headers::empty(), BodyString("Not found.")))
        }
    }
}
