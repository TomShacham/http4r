use std::time::{Instant};
use crate::handler::Handler;
use crate::httphandler::HttpHandler;
use crate::httpmessage::{Request, Response};

pub struct LoggingHttpHandler {
    pub log: Vec<String>,
    pub next_handler: HttpHandler
}

impl LoggingHttpHandler {
    pub fn new(next: HttpHandler) -> LoggingHttpHandler {
        LoggingHttpHandler {
            log: vec!(),
            next_handler: next
        }
    }
}

impl Handler for LoggingHttpHandler {
     fn handle<F>(self: &mut LoggingHttpHandler, req: Request, fun: F) -> ()
         where F: FnOnce(Response) -> () + Sized {
        let start = Instant::now();
        let req_string = format!("{} to {}", req.method.value().to_string(), req.uri);
        let response = (self.next_handler)(req);
        self.log.push(format!("{} => {} took {} μs", req_string, response.status.to_string(), start.elapsed().as_micros()));
        println!("{}", self.log.join("\n"));
        fun(response)
    }
}