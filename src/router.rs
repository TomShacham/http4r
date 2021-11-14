use crate::httphandler::HttpHandler;
use crate::httpmessage;
use crate::httpmessage::{Request, Response};

pub struct Router {}

impl HttpHandler for Router {
    fn handle(&self, req: Request) -> Response {
        match req.uri.as_str() {
            "/" => httpmessage::ok(vec!(), "".to_string()),
            _ => httpmessage::not_found(vec!(), "Not found".to_string()),
        }
    }
}
