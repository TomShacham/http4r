use http4r_core::handler::Handler;
use http4r_core::headers::Headers;
use http4r_core::http_message::{Method, Request, Response};
use http4r_core::http_message::Body::BodyString;
use http4r_core::query::Query;
use http4r_core::uri::Uri;
use regex::Regex;

pub struct Router<H> where H: Handler {
    profile_router: H,
}

impl<H> Router<H> where H: Handler {
    pub fn new(next: H) -> Router<H> {
        Router {
            profile_router: next
        }
    }
}

impl<H> Handler for Router<H> where H: Handler {
    fn handle<F>(&mut self, req: Request, fun: F) -> () where F: FnOnce(Response) -> () + Sized {
        match req {
            // redirect http traffic
            Request { uri: Uri { scheme: Some("http"), .. }, .. } => {
                fun(Response::moved_permanently(
                    Headers::from(vec!(
                        ("Location", format!("https://base-url/{}", req.uri.path).as_str()))),
                    BodyString("Ok!")));
            }
            // allow delete on this path
            Request { method: Method::DELETE, uri: Uri { path: "/delete-is-ok", .. }, .. } => {
                fun(Response::ok(Headers::empty(), BodyString("Ok!")));
            }
            // do not allow delete on any other path
            Request { method: Method::DELETE, .. } => {
                fun(Response::bad_request(Headers::empty(), BodyString("Naughty!")));
            }
            _ => fun(Response::not_found(Headers::empty(), BodyString("Not found.")))
        }
    }
}

pub struct ProfileRouter;

impl Handler for ProfileRouter {
    fn handle<F>(&mut self, req: Request, fun: F) -> () where F: FnOnce(Response) -> () + Sized {
        let profile_regex = Regex::new("/site/([^/]+)/profile").unwrap();
        match req {
            Request { .. } if Query::from(req.uri.query).get("org").is_none() => {
                fun(Response::bad_request(
                    Headers::empty(),
                    BodyString("Expected query parameter \"org\".")))
            }
            Request { .. } if req.headers.get("friend").is_none() => {
                fun(Response::bad_request(
                    Headers::empty(),
                    BodyString("Expected header \"friend\".")))
            }
            // we dont need to match the regex and path as that's done by the router before
            Request { .. } => {
                let captures = profile_regex.captures(req.uri.path);
                let name = captures.unwrap().get(1).unwrap().as_str();
                let org = Query::from(req.uri.query).get("org").unwrap();
                let friends = req.headers.get("friend").unwrap();
                fun(Response::ok(
                    Headers::empty(),
                    BodyString(format!("{}->{}: {}", org, name, friends).as_str())));
            }
            // we don't need the not found case now, this is handled by the router before us
        }
    }
}