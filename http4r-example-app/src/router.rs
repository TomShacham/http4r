use http4r_core::handler::Handler;
use http4r_core::headers::Headers;
use http4r_core::http_message::{Request, Response};
use http4r_core::http_message::Body::BodyString;
use http4r_core::query::Query;
use http4r_core::uri::Uri;
use regex::Regex;

pub struct Router;
impl Router {
    pub fn new() -> Router {
        Router {}
    }
}
impl Handler for Router {
    fn handle<F>(&mut self, req: Request, fun: F) -> () where F: FnOnce(Response) -> () + Sized {
        match req {
            Request { .. } => {
                let profile_regex = Regex::new("/site/([^/]+)/profile");
                let captures = profile_regex.unwrap().captures(req.uri.path);
                let name = if captures.is_some() {
                    captures.unwrap().get(1).unwrap().as_str()
                } else { "tom" };
                let org = Query::from(req.uri.query)
                fun(Response::ok(Headers::empty(), BodyString(name)));
            }
        }
    }
}
