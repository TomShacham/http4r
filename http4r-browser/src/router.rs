use http4r_core::handler::Handler;
use http4r_core::headers::Headers;
use http4r_core::http_message;
use http4r_core::http_message::{Body, Request, Response};
use http4r_core::http_message::Body::BodyString;
use http4r_core::uri::Uri;

pub struct Router {}

impl Handler for Router {
    fn handle<F>(&mut self, req: Request, fun: F) -> () where F: FnOnce(Response) -> () + Sized {
        match req.uri.to_string().as_str() {
            "/" => {
                fun(Response::ok(req.headers, BodyString("<h1>title</h1>")))
            },
            _ => fun(Response::not_found(Headers::empty(), BodyString("Not found"))),
        }
    }
}

pub struct NicerRouter {}

impl Handler for NicerRouter {
    fn handle<F>(&mut self, req: Request, fun: F) -> () where F: FnOnce(Response) -> () + Sized {
        match req {
            Request{ uri: Uri {path: "/some/path", .. }, .. } =>  {
                fun(Response::ok(Headers::empty(), BodyString(req.method.value().as_str())));
            }
            _ => fun(Response::not_found(Headers::empty(), Body::empty()))
        }
    }
}