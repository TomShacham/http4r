use crate::httpmessage::{Request, Response};

pub trait HttpHandler {
    fn handle(&self, req: Request) -> Response;
}

pub struct Handler<T>
    where T: Fn(Request) -> Response {
    pub(crate) handler: T,
}

impl<T> HttpHandler for Handler<T>
    where T: Fn(Request) -> Response {
    fn handle(&self, req: Request) -> Response {
        (self.handler)(req)
    }
}


impl<T> Handler<T>
    where T: Fn(Request) -> Response
{
    fn handle(&self, req: Request) -> Response {
        (self.handler)(req)
    }
}
