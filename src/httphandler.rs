use crate::httpmessage::{ok, Request, Response};

pub type HttpHandler = fn(Request) -> Response;

fn foo () {
    let x: HttpHandler = {
        |req: Request| { ok(vec!(), "".to_string())}
    };
}
