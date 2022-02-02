mod common;

#[cfg(test)]
mod tests {
    use http4r_core::client::Client;
    use http4r_core::handler::Handler;
    use http4r_core::headers::Headers;
    use http4r_core::http_message::Body::BodyString;
    use http4r_core::http_message::{body_string, Request};
    use http4r_core::http_message::Status::OK;
    use http4r_core::server::Server;
    use http4r_core::uri::Uri;
    use crate::common::{PassThroughHandler};

    #[test]
    fn encode_body_using_accept_encoding_prefers_brotli() {
        let mut server = Server::new(0);
        server.test(|| { Ok(PassThroughHandler {}) });

        let mut client = Client::new("127.0.0.1", server.port, None);
        let headers = Headers::from(vec!(("Accept-Encoding", "br, gzip, deflate")));
        let body = "Some quite long body".repeat(1000);

        let request = Request::post(
            Uri::parse("/"),
            headers,
            BodyString(body.as_str()));

        client.handle(request, |res| {
            assert_eq!(res.status, OK);
            assert_eq!(body, body_string(res.body));
            assert_eq!(res.headers.vec, vec!(
                ("Content-Encoding".to_string(), "br".to_string()),
                ("Content-Length".to_string(), "32".to_string())
            ));
        })
    }
}