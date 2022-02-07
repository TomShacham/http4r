use std::fs;
use std::fs::File;
use std::io::Read;
use std::str::from_utf8;
use http4r_core::handler::Handler;
use http4r_core::headers::Headers;
use http4r_core::http_message::{Request, Response};
use http4r_core::http_message::Body::BodyString;
use http4r_core::uri::Uri;

pub struct App<H> where H: Handler {
    handler: H,
}

impl<H> App<H> where H: Handler {
    pub fn new(handler: H) -> App<H> where H: Handler {
        App {
            handler
        }
    }
}

impl<H> Handler for App<H> where H: Handler {
    fn handle<F>(&mut self, req: Request, fun: F) -> () where F: FnOnce(Response) -> () + Sized {
        self.handler.handle(req, |res| {
            fun(res)
        })
    }
}
