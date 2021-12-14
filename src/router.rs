use crate::httpmessage;
use crate::httpmessage::{Request, Response};
use crate::httpmessage::Body::BodyString;

pub struct Router {}

impl Router {
    pub fn handle(&self, req: Request) -> Response {
        match req.uri.as_str() {
            "/" => httpmessage::ok(vec!(), BodyString("".to_string())),
            _ => httpmessage::not_found(vec!(), BodyString("Not found".to_string())),
        }
    }
}
