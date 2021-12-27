use crate::handler::Handler;
use crate::http_message::{moved_permanently, Request, Response};
use crate::http_message::Body::BodyString;

pub struct RedirectToHttpsHandler<H> where H: Handler {
    next_handler: H,
}

impl<H> Handler for RedirectToHttpsHandler<H> where H: Handler {
    fn handle<F>(self: &mut RedirectToHttpsHandler<H>, req: Request, fun: F) -> ()
        where F: FnOnce(Response) -> () + Sized {
        if req.uri.starts_with("http:") {
            let redirect = moved_permanently(vec!(("Location".to_string(), req.uri.replace("http", "https"))), BodyString("".to_string()));
            return fun(redirect);
        }
        self.next_handler.handle(req, fun);
    }
}

impl<H> RedirectToHttpsHandler<H> where H: Handler {
    pub fn new(handler: H) -> RedirectToHttpsHandler<H> {
        RedirectToHttpsHandler {
            next_handler: handler
        }
    }
}
