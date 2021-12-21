use crate::httpmessage::{moved_permanently, Request, Response};
use crate::httpmessage::Body::BodyString;

pub trait Handler {
    fn handle<F>(&mut self, req: Request, fun: F) -> ()
        where F: FnOnce(Response) -> () + Sized;
}

