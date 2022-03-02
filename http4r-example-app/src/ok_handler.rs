use http4r_core::handler::Handler;
use http4r_core::headers::Headers;
use http4r_core::http_message::{Request, Response};
use http4r_core::http_message::Body::BodyString;
use http4r_core::uri::Uri;

pub struct OkHandler;

impl Handler for OkHandler {
    fn handle<F>(&mut self, req: Request, fun: F) -> () where F: FnOnce(Response) -> () + Sized {
        match req {
            Request { uri: Uri { path: "/", .. }, .. } => {
                fun(Response::ok(Headers::empty(), BodyString("Hello, world!")));
            }
            _ => {
                fun(Response::not_found(Headers::empty(), BodyString("Not found.")));
            }
        }
    }
}