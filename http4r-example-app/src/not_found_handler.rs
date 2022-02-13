use std::fs;
use std::fs::canonicalize;
use http4r_core::handler::Handler;
use http4r_core::headers::Headers;
use http4r_core::http_message::{Request, Response};
use http4r_core::http_message::Body::BodyString;
use http4r_core::http_message::Status::{Forbidden, NotFound};

pub struct NotFoundHandler<H> where H: Handler {
    handler: H,
}

impl<H> NotFoundHandler<H> where H: Handler {
    pub fn new(handler: H) -> NotFoundHandler<H> {
        NotFoundHandler {
            handler
        }
    }
}

impl<H> Handler for NotFoundHandler<H> where H: Handler {
    fn handle<F>(&mut self, req: Request, fun: F) -> () where F: FnOnce(Response) -> () + Sized {
        self.handler.handle(req, |res| {
            if res.status == NotFound || res.status == Forbidden {
                let result = canonicalize("/resources/html/not-found.html");
                if result.is_err() {
                    fun(Response::not_found(Headers::empty(), BodyString("Not found.")));
                    return
                }
                let canonical = result.unwrap();
                println!("opening {}", canonical.to_str().unwrap());
                let not_found = fs::read_to_string(canonical.to_str().unwrap()).unwrap();
                fun(Response::not_found(Headers::empty(), BodyString(not_found.as_str())))
            } else {
                fun(res)
            }
        })
    }
}