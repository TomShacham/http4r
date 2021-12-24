use rusty::httpmessage::Status::{NotFound, OK};
use rusty::handler::Handler;
use rusty::httpmessage::{not_found, ok, Request, Response};
use rusty::httpmessage::Body::BodyString;

#[cfg(test)]
mod tests {
    use std::io::{Read, repeat};
    use rusty::client::Client;
    use rusty::httphandler::HttpHandler;
    use rusty::httpmessage::{body_string, get, headers_to_string, not_found, ok, post, Response};
    use rusty::httpmessage::Body::{BodyStream, BodyString};
    use rusty::logging_handler::LoggingHttpHandler;
    use rusty::redirect_to_https_handler::RedirectToHttpsHandler;
    use rusty::server::{Server, ServerOptions};
    use super::*;

    #[test]
    fn client_over_http_get() {
        let port = 7878;
        Server::new(||{Ok(PassThroughHandler {})}, ServerOptions { port: Some(port), pool: None });
        let mut client = Client { base_uri: String::from("127.0.0.1"), port };
        let request = get("/".to_string(), vec!());

        client.handle(request, |response: Response| {
            assert_eq!("OK", response.status.to_string());
            assert_eq!("Content-Length: 0", headers_to_string(&response.headers));
            assert_eq!("".to_string(), body_string(response.body));
        });
    }

    #[test]
    fn gives_you_a_bodystream_if_entity_bigger_than_buffer() {
        let port = 7879;
        let buffer = repeat(116).take(20000);

        Server::new(||{Ok(PassThroughHandler {})}, ServerOptions { port: Some(port), pool: None });
        let mut client = Client { base_uri: String::from("127.0.0.1"), port };
        let post_with_stream_body = post("/".to_string(), vec!(("Content-Length".to_string(), 20000.to_string())), BodyStream(Box::new(buffer)));

        client.handle(post_with_stream_body, |response| {
            match response.body {
                BodyString(_) => panic!("Should not be BodyString"),
                BodyStream(s) => {
                    let string = body_string(BodyStream(s));
                    assert_eq!(20000, string.len());
                    assert_eq!(string[0..10], "tttttttttt".to_string());
                    assert_eq!("OK", response.status.to_string());
                    assert_eq!("Content-Length: 20000", headers_to_string(&response.headers));
                }
            }
        });
    }

    #[test]
    fn client_must_provide_content_length_or_else_transfer_encoding_is_chunked_if_entity_big() {}

    #[test]
    fn router_non_http() {
        let router: HttpHandler = |req| {
            match req.uri.as_str() {
                "/" => ok(vec!(), BodyString("".to_string())),
                _ => not_found(vec!(), BodyString("Not found".to_string())),
            }
        };
        let request = get("/".to_string(), vec!());
        let request_to_no_route = get("no/route/here".to_string(), vec!());

        assert_eq!(OK, router(request).status.into());
        assert_eq!(NotFound, router(request_to_no_route).status);
    }


    #[test]
    fn can_compose_http_handlers() {
        let router = Router{};
        let logger = LoggingHttpHandler::new(router);
        let mut redirector = RedirectToHttpsHandler::new(logger);

        let request = get("/".to_string(), vec!());
        let request_to_no_route = get("no/route/here".to_string(), vec!());

        // non-http
        redirector.handle(request, |response| {
            assert_eq!(OK, response.status.into());
        });
        redirector.handle(request_to_no_route, |response| {
            assert_eq!(NotFound, response.status);
        });

        //http
        let port = 7880;
        Server::new(|| Ok(RedirectToHttpsHandler::new(LoggingHttpHandler::new(Router{}))), ServerOptions {port: Some(port), pool: None});
        let mut client = Client { base_uri: String::from("127.0.0.1"), port };
        let request = get("/".to_string(), vec!());

        client.handle(request, |response: Response| {
            assert_eq!("OK", response.status.to_string());
            assert_eq!("Content-Length: 0", headers_to_string(&response.headers));
            assert_eq!("".to_string(), body_string(response.body));
        });
    }
}

struct Router {}

impl Handler for Router {
    fn handle<F>(&mut self, req: Request, fun: F) -> () where F: FnOnce(Response) -> () + Sized {
        let response = match req.uri.as_str() {
            "/" => ok(vec!(("Content-Length".to_string(), 0.to_string())), BodyString("".to_string())),
            _ => not_found(vec!(("Content-Length".to_string(), 9.to_string())), BodyString("Not found".to_string())),
        };
        fun(response)
    }
}

struct PassThroughHandler {}

impl Handler for PassThroughHandler {
    fn handle<F>(&mut self, req: Request, fun: F) -> () where F: FnOnce(Response) -> () + Sized {
            fun(ok(req.headers, req.body))
    }
}

//todo() DO NOT EXPECT A CONTENT LENGTH FOR HEAD,OPTIONS,CONNECT,204,1XX ETC
//todo() test zero length body
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
//todo() get doesnt send body etc - all the method semantics