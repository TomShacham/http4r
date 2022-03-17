mod common;

#[cfg(test)]
mod tests {
    use http4r_core::client::{Client, WithContentLength};
    use http4r_core::handler::Handler;
    use http4r_core::headers::Headers;
    use http4r_core::http_message::Body::BodyString;
    use http4r_core::http_message::Method::{CONNECT, GET, HEAD, OPTIONS, TRACE};
    use http4r_core::http_message::{body_string, Request, Response};
    use http4r_core::server::Server;
    use http4r_core::uri::Uri;
    use crate::common::PassThroughHandler;

    #[test]
    fn method_semantics_ignore_body_of_get_head_options_connect_trace() {
        let mut server = Server::new(0);
        server.start(|| { Ok(PassThroughHandler {}) }, true);

        let mut client = WithContentLength::new(
            Client::new("127.0.0.1", server.port, None)
        );

        let methods = vec!(GET, HEAD, OPTIONS, CONNECT, TRACE);

        for method in methods {
            let should_ignore_body = Request::request(method, Uri::parse("/"), Headers::empty())
                .with_body(BodyString("non empty body"));

            client.handle(should_ignore_body, |response: Response| {
                assert_eq!("OK", response.status.to_string());
                assert_eq!("".to_string(), body_string(response.body));
                assert_eq!(format!("Content-Length: 0\r\nHost: 127.0.0.1:{}", server.port), response.headers.to_wire_string());
            });
        }
    }

}

/*



todo()
Any response to a HEAD request and any response with a 1xx
       (Informational), 204 (No Content), or 304 (Not Modified) status
       code is always terminated by the first empty line after the
       header fields, regardless of the header fields present in the
       message, and thus cannot contain a message body.
 */
