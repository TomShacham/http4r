use rusty::httpmessage::Status::{NotFound, OK};

#[cfg(test)]
mod tests {
    use std::io::{BufRead, BufReader, Read, repeat};
    use rusty::client::Client;
    use rusty::httphandler::HttpHandler;
    use rusty::httpmessage::{body_string, get, header, headers_to_string, ok, post, Request};
    use rusty::httpmessage::Body::{BodyStream, BodyString};
    use rusty::router::Router;
    use rusty::server::{Server, ServerOptions};
    use super::*;

    #[test]
    fn client_over_http_get() {
        let port = 7878;
        let pass_through_req_handler: HttpHandler = {
            |req: Request| { return ok(req.headers, req.body); }
        };

        Server::new(pass_through_req_handler, ServerOptions { port: Some(port), pool: None });
        let client = Client { base_uri: String::from("127.0.0.1"), port };
        let request = get("".to_string(), vec!());
        let response = client.handle(request);

        assert_eq!("OK", response.status.to_string());
        assert_eq!("Content-Length: 0", headers_to_string(&response.headers));
        assert_eq!("".to_string(), body_string(response.body));
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
    #[test]
    fn gives_you_a_bodystream_if_entity_bigger_than_buffer() {
        let port = 7879;
        let pass_through_req_handler: HttpHandler = {
            |req: Request| { return ok(req.headers, req.body); }
        };
        let buffer = repeat(1).take(20000);

        Server::new(pass_through_req_handler, ServerOptions { port: Some(port), pool: None });
        let client = Client { base_uri: String::from("127.0.0.1"), port };
        let request = post("/".to_string(), vec!(("Content-Length".to_string(), 20000.to_string())), BodyStream(Box::new(buffer)));
        let response = client.handle(request);

        assert_eq!("OK", response.status.to_string());
        assert_eq!("Content-Length: 20000", headers_to_string(&response.headers));

        match response.body {
            BodyString(str) => panic!("Should not be BodyString"),
            BodyStream(str) => assert!(true)
        }
    }

    #[test]
    fn client_must_provide_content_length_if_entity_big() {}

    #[test]
    fn router_non_http() {
        let router = Router {};
        let request = get("/".to_string(), vec!());
        let request_to_no_route = get("no/route/here".to_string(), vec!());

        assert_eq!(OK, router.handle(request).status.into());
        assert_eq!(NotFound, router.handle(request_to_no_route).status);
    }
}