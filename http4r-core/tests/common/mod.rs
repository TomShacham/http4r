use http4r_core::handler::Handler;
use http4r_core::headers::Headers;
use http4r_core::http_message::{Body, Request, Response};
use http4r_core::http_message::Body::BodyString;

pub struct Router {}

impl Handler for Router {
    fn handle<F>(&mut self, req: Request, fun: F) -> () where F: FnOnce(Response) -> () + Sized {
        let response = match req.uri.to_string().as_str() {
            "/" => Response::ok(Headers::empty(), Body::empty()),
            _ => Response::not_found(Headers::empty(), BodyString("Not found")),
        };
        fun(response)
    }
}

pub struct PassThroughHandler {}

impl Handler for PassThroughHandler {
    fn handle<F>(&mut self, req: Request, fun: F) -> () where F: FnOnce(Response) -> () + Sized {
        let response = if req.trailers.is_empty() {
            Response::ok(req.headers, req.body)
        } else {
            Response::ok(req.headers, req.body).with_trailers(req.trailers)
        };
        fun(response);
    }
}