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
            Request { uri: Uri { path: "/root", .. }, .. } => {
                fun(Response::ok(Headers::empty(), BodyString("hello, world")))
            }
            _ => fun(Response::not_found(Headers::empty(), BodyString("Not found.")))
        }
    }
}
