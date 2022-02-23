use http4r_core::http_message::Status::{NotFound, OK};
use http4r_core::handler::Handler;

mod common;

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use http4r_core::client::{Client};
    use http4r_core::headers::Headers;
    use http4r_core::http_message::{body_string, Request, Response};
    use http4r_core::http_message::Body::{BodyStream, BodyString};
    use http4r_core::logging_handler::{LoggingHttpHandler, RustLogger, WasmClock};
    use http4r_core::redirect_to_https_handler::RedirectToHttpsHandler;
    use http4r_core::server::Server;
    use http4r_core::uri::Uri;
    use crate::common::{PassThroughHandler, Router};
    use super::*;

    #[test]
    fn client_over_http_get() {
        let mut server = Server::new(0);
        server.start(|| { Ok(PassThroughHandler {}) }, true);
        let mut client = Client::new("127.0.0.1", server.port, None);
        let request = Request::get(Uri::parse("/"), Headers::empty());

        client.handle(request, |response: Response| {
            assert_eq!("OK", response.status.to_string());
            assert_eq!("Content-Length: 0", response.headers.to_wire_string());
            assert_eq!("".to_string(), body_string(response.body));
        });
    }

    #[test]
    fn gives_you_a_bodystream_if_entity_bigger_than_buffer() {
        let long_string = "t".repeat(20000);

        let mut server = Server::new(0);
        server.start(|| { Ok(PassThroughHandler {}) }, true);
        let mut client = Client::new("127.0.0.1", server.port, None);
        let post_with_stream_body = Request::post(
            Uri::parse("/"),
            Headers::from(vec!(("Content-Length", "20000"))),
            BodyString(long_string.as_str()));

        client.handle(post_with_stream_body, |response| {
            match response.body {
                BodyString(_) => panic!("Should not be BodyString"),
                BodyStream(s) => {
                    let string = body_string(BodyStream(s));
                    assert_eq!(20000, string.len());
                    assert_eq!(string, long_string.to_string());
                    assert_eq!("OK", response.status.to_string());
                    assert_eq!("Content-Length: 20000", response.headers.to_wire_string());
                }
            }
        });
    }

    #[test]
    fn can_handle_no_headers() {
        let mut server = Server::new(0);
        server.start(|| { Ok(PassThroughHandler {}) }, true);
        let mut client = Client::new("127.0.0.1", server.port, None);
        let no_headers = Request::get(Uri::parse("/"), Headers::empty());

        client.handle(no_headers, |response: Response| {
            assert_eq!("OK", response.status.to_string());
            assert_eq!("Content-Length: 0", response.headers.to_wire_string());
            assert_eq!("".to_string(), body_string(response.body));
        });
    }

    #[test]
    fn can_compose_http_handlers() {
        let router = Router {};
        let logger = LoggingHttpHandler::new(RustLogger {}, WasmClock {}, router);
        let mut redirector = RedirectToHttpsHandler::new(logger, HashMap::new());

        let request = Request::get(Uri::parse("/"), Headers::empty());
        let request_to_no_route = Request::get(Uri::parse("no/route/here"), Headers::empty());

        // non-http
        redirector.handle(request, |response| {
            assert_eq!(OK, response.status.into());
        });
        redirector.handle(request_to_no_route, |response| {
            assert_eq!(NotFound, response.status);
        });

        //http
        let mut server = Server::new(0);
        server.start(|| Ok(RedirectToHttpsHandler::new(LoggingHttpHandler::new(RustLogger {}, WasmClock {}, Router {}), HashMap::new())), true);
        let mut client = Client::new("127.0.0.1", server.port, None);
        let request = Request::get(Uri::parse("/"), Headers::empty());

        client.handle(request, |response: Response| {
            assert_eq!("OK", response.status.to_string());
            assert_eq!("Content-Length: 0", response.headers.to_wire_string());
            assert_eq!("".to_string(), body_string(response.body));
        });
    }
}


//todo() DO NOT EXPECT A CONTENT LENGTH FOR HEAD,OPTIONS,CONNECT,204,1XX ETC
//todo() handle set-cookie especially as multiple headers of this value cannot be combined
// with commas
//todo() allow header for 405 method not allowed
