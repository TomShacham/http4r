use http4r_core::handler::Handler;
use http4r_core::headers::Headers;
use http4r_core::http_message::{Request, Response};
use http4r_core::http_message::Body::BodyString;

pub struct OkHandler;
impl Handler for OkHandler {
    fn handle<F>(&mut self, _req: Request, fun: F) -> () where F: FnOnce(Response) -> () + Sized {
        fun(Response::not_found(Headers::empty(), BodyString("Not found.")));
    }
}