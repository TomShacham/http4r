use instant::Instant;
use wasm_bindgen::JsValue;
use crate::handler::Handler;
use crate::http_message::{Request, Response};
use web_sys::{console};

pub struct LoggingHttpHandler<H, C, L> where H: Handler, C: Clock, L: Logger {
    pub log: Vec<String>,
    pub next_handler: H,
    pub clock: C,
    pub logger: L
}

pub trait Clock {
    fn now(&mut self) -> Instant;
}

pub trait Logger {
    fn log(&mut self, line: &str);
}

pub struct ConsoleLogger {}
impl Logger for ConsoleLogger {
    fn log(&mut self, line: &str) {
        let array = js_sys::Array::from(&JsValue::from_str(line));
        console::log(&array)
    }
}

pub struct RustLogger {}
impl Logger for RustLogger {
    fn log(&mut self, line: &str) {
        println!("{}", line)
    }
}

pub struct WasmClock {}
impl Clock for WasmClock {
    fn now(&mut self) -> Instant {
        instant::Instant::now()
    }
}

impl<H, C, L> LoggingHttpHandler<H, C, L> where H: Handler, C: Clock, L: Logger {
    pub fn new(logger: L, clock: C, next: H) -> LoggingHttpHandler<H, C, L> {
        LoggingHttpHandler {
            log: vec!(),
            next_handler: next,
            clock,
            logger
        }
    }
}

impl<H, C, L> Handler for LoggingHttpHandler<H, C, L> where H: Handler, C: Clock, L: Logger {
     fn handle<F>(self: &mut LoggingHttpHandler<H, C, L>, req: Request, fun: F) -> ()
         where F: FnOnce(Response) -> () + Sized {
        let start = self.clock.now();
        let req_string = format!("{} to {}", req.method.value().to_string(), req.uri);
        self.next_handler.handle(req, |res| {
            let status = res.status.to_string();
            fun(res);
            self.log.push(format!("{} => {} took {} μs", req_string, status, start.elapsed().as_micros()));
            self.logger.log(format!("{}", self.log.join("\n")).as_str());
        });
    }
}