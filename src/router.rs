use crate::http_message;
use crate::http_message::{Request, Response};
use crate::http_message::Body::BodyString;

pub struct Router {}

impl Router {
    pub fn handle(&self, req: Request) -> Response {
        match req.uri.as_str() {
            "/" => http_message::ok(vec!(), BodyString("".to_string())),
            _ => http_message::not_found(vec!(), BodyString("Not found".to_string())),
        }
    }
}
