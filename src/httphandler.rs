use crate::httpmessage::{Request, Response};

pub type HttpHandler = fn(Request) -> Response;
