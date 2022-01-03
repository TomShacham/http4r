use wasm_bindgen::prelude::*;
use std::panic;
use web_sys::{console};
use wasm_bindgen::JsValue;
use http4r_core::handler::Handler;
use http4r_core::headers::Headers;
use http4r_core::http_message::{body_string, HttpVersion, Method, one_pt_one, Request, Response};
use http4r_core::http_message::Body::BodyString;
use http4r_core::logging_handler::{Logger, LoggingHttpHandler, WasmClock};
use http4r_core::uri::Uri;
use crate::router::Router;

#[wasm_bindgen]
pub struct JSRequest {
    uri: String,
    body: String,
    method: String,
    headers: String,
}

#[allow(non_snake_case)]
#[wasm_bindgen]
pub fn jsRequest(method: &str, uri: &str, body: &str, headers: &str) -> JSRequest {
    let headers = headers.split("; ").fold(vec!(), |mut acc: Vec<(String, String)>, next: &str| {
        let mut split = next.split(": ");
        acc.push((split.next().unwrap().to_string(), split.next().unwrap().to_string()));
        acc
    });
    return JSRequest {
        method: method.to_string(),
        uri: uri.to_string(),
        body: body.to_string(),
        headers: Headers::js_headers_to_string(&headers),
    };
}

#[wasm_bindgen]
pub struct JSResponse {
    body: String,
    status: u32,
    headers: String,
}

#[wasm_bindgen]
impl JSResponse {
    pub fn body(&self) -> String {
        (&self.body).to_string()
    }
    pub fn status(&self) -> String {
        (&self.status).to_string()
    }
    pub fn headers(&self) -> String {
        (&self.headers).to_string()
    }
}

#[wasm_bindgen]
pub fn serve(req: JSRequest) -> JSResponse {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    let mut app = ExampleApp::new(LoggingHttpHandler::new(ConsoleLogger {}, WasmClock {}, Router {}));
    let request = Request {
        headers: Headers::js_headers_from_string(&req.headers),
        method: Method::from(&req.method),
        uri: Uri::parse(&req.uri),
        body: BodyString(req.body.as_str()),
        version: one_pt_one()
    };
    let mut response = JSResponse {
        body: "Not found".to_string(),
        headers: "Content-Type: text/plain".to_string(),
        status: 404,
    };
    app.handle(request, |res| {
        response = JSResponse {
            body: body_string(res.body),
            status: res.status.value(),
            headers: Headers::js_headers_to_string(&res.headers.vec),
        }
    });
    response
}

pub struct ExampleApp<H> where H: Handler {
    next_handler: H,
}

impl<H> ExampleApp<H> where H: Handler {
    pub fn new(next_handler: H) -> ExampleApp<H> {
        ExampleApp {
            next_handler
        }
    }
}

impl<H> Handler for ExampleApp<H> where H: Handler {
    fn handle<F>(&mut self, req: Request, fun: F) -> () where F: FnOnce(Response) -> () + Sized {
        println!("App start");
        self.next_handler.handle(req, |res| {
            fun(res);
            println!("App end")
        })
    }
}


pub struct ConsoleLogger {}

impl Logger for ConsoleLogger {
    fn log(&mut self, line: &str) {
        let array = js_sys::Array::from(&JsValue::from_str(line));
        console::log(&array)
    }
}