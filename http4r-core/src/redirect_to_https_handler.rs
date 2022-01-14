use std::collections::HashMap;
use crate::handler::Handler;
use crate::headers::Headers;
use crate::http_message::{Body, Request, Response};

type Env = HashMap<String, String>;

pub struct RedirectToHttpsHandler<H> where H: Handler {
    next_handler: H,
    env: Env,
}

impl<H> Handler for RedirectToHttpsHandler<H> where H: Handler {
    fn handle<'a, F>(self: &mut RedirectToHttpsHandler<H>, req: Request<'a>, fun: F) -> ()
        where F: FnOnce(Response) -> () + Sized {
        if self.env.get("environment") == Some(&"production".to_string()) && req.uri.scheme != Some("https") {
            let redirect = Response::moved_permanently(
                Headers::from(vec!(("Location", req.uri.with_scheme("https").to_string().as_str()))),
                Body::empty());
            return fun(redirect);
        }
        self.next_handler.handle(req, fun);
    }
}

impl<H> RedirectToHttpsHandler<H> where H: Handler {
    pub fn new(handler: H, env: Env) -> RedirectToHttpsHandler<H> {
        RedirectToHttpsHandler {
            next_handler: handler,
            env,
        }
    }
}
