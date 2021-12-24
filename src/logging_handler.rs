use std::time::{Instant};
use crate::handler::Handler;
use crate::http_message::{Request, Response};

pub struct LoggingHttpHandler<H> where H: Handler {
    pub log: Vec<String>,
    pub next_handler: H
}

impl<H> LoggingHttpHandler<H> where H: Handler {
    pub fn new(next: H) -> LoggingHttpHandler<H> {
        LoggingHttpHandler {
            log: vec!(),
            next_handler: next
        }
    }
}

impl<H> Handler for LoggingHttpHandler<H> where H: Handler {
     fn handle<F>(self: &mut LoggingHttpHandler<H>, req: Request, fun: F) -> ()
         where F: FnOnce(Response) -> () + Sized {
        let start = Instant::now();
        let req_string = format!("{} to {}", req.method.value().to_string(), req.uri);
        self.next_handler.handle(req, |res| {
            let status = res.status.to_string();
            fun(res);
            self.log.push(format!("{} => {} took {} μs", req_string, status, start.elapsed().as_micros()));
            println!("{}", self.log.join("\n"));
        });
    }
}