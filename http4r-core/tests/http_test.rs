use http4r_core::http_message::Status::{NotFound, OK};
use http4r_core::handler::Handler;
use http4r_core::headers::Headers;
use http4r_core::http_message::Body::BodyString;
use http4r_core::http_message::{Request, Response};

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::io::{Read, repeat};
    use http4r_core::client::{Client, WithContentLength};
    use http4r_core::headers::Headers;
    use http4r_core::http_message::{body_string, Request, Response};
    use http4r_core::http_message::Body::{BodyStream, BodyString};
    use http4r_core::http_message::Method::{CONNECT, GET, HEAD, OPTIONS, TRACE};
    use http4r_core::http_message::Status::BadRequest;
    use http4r_core::logging_handler::{LoggingHttpHandler, RustLogger, WasmClock};
    use http4r_core::redirect_to_https_handler::RedirectToHttpsHandler;
    use http4r_core::server::Server;
    use http4r_core::uri::Uri;
    use super::*;

    #[test]
    fn client_over_http_get() {
        let mut server = Server::new(0);
        server.test(|| { Ok(PassThroughHandler {}) });
        let mut client = Client { base_uri: String::from("127.0.0.1"), port: server.port };
        let request = Request::get(Uri::parse("/"), Headers::empty());

        client.handle(request, |response: Response| {
            assert_eq!("OK", response.status.to_string());
            assert_eq!("Content-Length: 0", response.headers.to_wire_string());
            assert_eq!("".to_string(), body_string(response.body));
        });
    }

    #[test]
    fn gives_you_a_bodystream_if_entity_bigger_than_buffer() {
        let buffer = repeat(116).take(20000);

        let mut server = Server::new(0);
        server.test(|| { Ok(PassThroughHandler {}) });
        let mut client = Client { base_uri: String::from("127.0.0.1"), port: server.port };
        let post_with_stream_body = Request::post(Uri::parse("/"), Headers::from(vec!(("Content-Length", "20000"))), BodyStream(Box::new(buffer)));

        client.handle(post_with_stream_body, |response| {
            match response.body {
                BodyString(_) => panic!("Should not be BodyString"),
                BodyStream(s) => {
                    let string = body_string(BodyStream(s));
                    assert_eq!(20000, string.len());
                    assert_eq!(string[0..10], "tttttttttt".to_string());
                    assert_eq!("OK", response.status.to_string());
                    assert_eq!("Content-Length: 20000", response.headers.to_wire_string());
                }
            }
        });
    }

    #[test]
    fn can_handle_no_headers() {
        let mut server = Server::new(0);
        server.test(|| { Ok(PassThroughHandler {}) });
        let mut client = Client { base_uri: String::from("127.0.0.1"), port: server.port };
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
        server.test(|| Ok(RedirectToHttpsHandler::new(LoggingHttpHandler::new(RustLogger {}, WasmClock {}, Router {}), HashMap::new())));
        let mut client = Client { base_uri: String::from("127.0.0.1"), port: server.port };
        let request = Request::get(Uri::parse("/"), Headers::empty());

        client.handle(request, |response: Response| {
            assert_eq!("OK", response.status.to_string());
            assert_eq!("Content-Length: 0", response.headers.to_wire_string());
            assert_eq!("".to_string(), body_string(response.body));
        });
    }

    #[test]
    fn method_semantics_ignore_body_of_get_head_options_connect_trace() {
        let mut server = Server::new(0);
        server.test(|| { Ok(PassThroughHandler {}) });

        let mut client = WithContentLength::new(
            Client { base_uri: String::from("127.0.0.1"), port: server.port }
        );

        let methods = vec!(GET, HEAD, OPTIONS, CONNECT, TRACE);

        for method in methods {
            let should_ignore_body = Request::request(method, Uri::parse("/"), Headers::empty())
                .with_body(BodyString("non empty body"));

            client.handle(should_ignore_body, |response: Response| {
                assert_eq!("OK", response.status.to_string());
                assert_eq!("".to_string(), body_string(response.body));
                assert_eq!("Content-Length: 0", response.headers.to_wire_string());
            });
        }
    }

    #[test]
    fn duplicate_different_content_length_headers_result_in_bad_request() {

        let mut server = Server::new(0);
        server.test(|| { Ok(PassThroughHandler {}) });

        let mut client = WithContentLength::new(
            Client { base_uri: String::from("127.0.0.1"), port: server.port }
        );

        client.handle( Request::post(Uri::parse("/bob"), Headers::from(
            vec!( ("Content-length", "10"), ("Content-length", "20") )
        ), BodyString("hello")), | response: Response | {
            assert_eq!(BadRequest, response.status);
            assert_eq!("", body_string(response.body));
        })
    }
}

struct Router {}

impl Handler for Router {
    fn handle<F>(&mut self, req: Request, fun: F) -> () where F: FnOnce(Response) -> () + Sized {
        let response = match req.uri.to_string().as_str() {
            "/" => Response::ok(Headers::empty(), BodyString("")),
            _ => Response::not_found(Headers::empty(), BodyString("Not found")),
        };
        fun(response)
    }
}

struct PassThroughHandler {}

impl Handler for PassThroughHandler {
    fn handle<F>(&mut self, req: Request, fun: F) -> () where F: FnOnce(Response) -> () + Sized {
        fun(Response::ok(req.headers, req.body))
    }
}

//todo() DO NOT EXPECT A CONTENT LENGTH FOR HEAD,OPTIONS,CONNECT,204,1XX ETC
//todo() handle duplicate Content-lengths Content-Length: 42, 42
//todo() handle both transfer encoding and content length as error
// If a message is received with both a Transfer-Encoding and a
//        Content-Length header field, the Transfer-Encoding overrides the
//        Content-Length.  Such a message might indicate an attempt to
//        perform request smuggling (Section 9.5) or response splitting
//        (Section 9.4) and ought to be handled as an error.  A sender MUST
//        remove the received Content-Length field prior to forwarding such
//        a message downstream.
//todo() If a message is received without Transfer-Encoding and with
//        either multiple Content-Length header fields having differing
//        field-values or a single Content-Length header field having an
//        invalid value, then the message framing is invalid and the
//        recipient MUST treat it as an unrecoverable error.  If this is a
//        request message, the server MUST respond with a 400 (Bad Request)
//        status code and then close the connection.
//todo() When a server listening only for HTTP request messages, or processing
//    what appears from the start-line to be an HTTP request message,
//    receives a sequence of octets that does not match the HTTP-message
//    grammar aside from the robustness exceptions listed above, the server
//    SHOULD respond with a 400 (Bad Request) response.
//todo() handle set-cookie especially as multiple headers of this value cannot be combined
// with commas
//todo() allow header for 405 method not allowed