use http4r_core::handler::Handler;
use http4r_core::headers::Headers;
use http4r_core::http_message;
use http4r_core::http_message::{Request, Response};
use http4r_core::http_message::Body::BodyString;

pub struct Router {}

impl Handler for Router {
    fn handle<F>(&mut self, req: Request, fun: F) -> () where F: FnOnce(Response) -> () + Sized {
        match req.uri.as_str() {
            "/" => {
                fun(http_message::ok(req.headers, BodyString("<h1>title</h1>")))
            },
            _ => fun(http_message::not_found(Headers::empty(), BodyString("Not found"))),
        }
    }
}
