use crate::http_message::{Request, Response};

pub trait Handler {
    fn handle<F>(&mut self, req: Request, fun: F) -> ()
        where F: FnOnce(Response) -> () + Sized;
}

