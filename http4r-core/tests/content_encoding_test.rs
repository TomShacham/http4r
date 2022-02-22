mod common;

#[cfg(test)]
mod tests {
    use http4r_core::client::Client;
    use http4r_core::handler::Handler;
    use http4r_core::headers::Headers;
    use http4r_core::http_message::Body::{BodyStream, BodyString};
    use http4r_core::http_message::{body_string, Request};
    use http4r_core::http_message::Status::OK;
    use http4r_core::server::Server;
    use http4r_core::uri::Uri;
    use crate::common::{EchoBodyHandler, PassHeadersAsBody, PassThroughHandler, SetContentEncodingToNoneAndEchoHeaders};

    #[test]
    fn encode_body_using_accept_encoding_prefers_brotli() {
        let mut server = Server::new(0);
        server.start(|| { Ok(PassThroughHandler {}) });

        let mut client = Client::new("127.0.0.1", server.port, None);
        let headers = Headers::from(vec!(("Accept-Encoding", "gzip, deflate, br")));
        let body = "Some quite long body".repeat(1000);

        let request = Request::post(
            Uri::parse("/"),
            headers,
            BodyString(body.as_str()));

        client.handle(request, |res| {
            assert_eq!(res.status, OK);
            assert_eq!(body, body_string(res.body));
            assert_eq!(res.headers.vec, vec!(
                ("Accept-Encoding".to_string(), "gzip, deflate, br".to_string()),
                ("Content-Length".to_string(), "20000".to_string()),
                ("Content-Encoding".to_string(), "br".to_string()),
            ));
        })
    }

    #[test]
    fn client_can_send_a_compressed_message() {
        let mut server = Server::new(0);
        server.start(|| { Ok(PassThroughHandler {}) });

        let mut client = Client::new("127.0.0.1", server.port, None);
        let headers = Headers::from(vec!(("Content-Encoding", "br")));
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
                ("Content-Length".to_string(), "20000".to_string()),
            ));
        })

    }

    #[test]
    fn compressed_body_stream() {
        let mut server = Server::new(0);
        server.start(|| { Ok(PassThroughHandler {}) });

        let mut client = Client::new("127.0.0.1", server.port, None);
        let headers = Headers::from(vec!(
            ("Content-Encoding", "br"),
            ("Accept-Encoding", "br"),
        ));
        let body = "Some quite long body".repeat(1000);

        let request = Request::post(
            Uri::parse("/"),
            headers,
            BodyStream(Box::new(body.as_bytes())));

        client.handle(request, |res| {
            assert_eq!(body, body_string(res.body));
            assert_eq!(res.status, OK);
            assert_eq!(res.headers.vec, vec!(
                ("Content-Encoding".to_string(), "br".to_string()),
                ("Accept-Encoding".to_string(), "br".to_string()),
                ("Transfer-Encoding".to_string(), "brotli, chunked".to_string()),
            ));
        })
    }

    #[test]
    fn accept_encoding_wins_over_transfer_encoding_as_client_may_send_in_one_format_and_accept_in_another() {
        let mut server = Server::new(0);
        server.start(|| { Ok(PassHeadersAsBody {}) });

        let mut client = Client::new("127.0.0.1", server.port, None);
        let headers = Headers::from(vec!(
            ("Transfer-Encoding", "gzip, chunked"),
            ("Accept-Encoding", "br, gzip, deflate"),
            ("Content-Encoding", "br"),
        ));
        let expected_body = Headers::from_headers(&headers);

        let str = "Some body";
        let request = Request::post(
            Uri::parse("/"),
            headers,
            BodyStream(Box::new(str.as_bytes())));

        client.handle(request, move|res| {
            assert_eq!(expected_body.to_wire_string(), body_string(res.body));
            assert_eq!(res.status, OK);
            assert_eq!(res.headers.vec, vec!(
                ("Content-Length".to_string(), "90".to_string()),
                ("Content-Encoding".to_string(), "br".to_string()),
            ));
        })
    }

    #[test]
    fn setting_response_header_of_content_encoding_none_wins_over_everything() {
        let mut server = Server::new(0);
        server.start(|| { Ok(SetContentEncodingToNoneAndEchoHeaders {}) });

        let mut client = Client::new("127.0.0.1", server.port, None);
        let headers = Headers::from(vec!(
            ("Transfer-Encoding", "gzip, chunked"),
            ("Accept-Encoding", "br, gzip, deflate"),
            ("Content-Encoding", "br"),
        ));
        let expected_body = Headers::from_headers(&headers);

        let str = "Some body";
        let request = Request::post(
            Uri::parse("/"),
            headers,
            BodyStream(Box::new(str.as_bytes())));

        client.handle(request, move|res| {
            assert_eq!(res.status, OK);
            assert_eq!(res.headers.vec, vec!(
               // no content encoding
               ("Content-Length".to_string(), "0".to_string())
            ));
        })
    }

    #[test]
    fn transfer_encoding_wins_over_content_encoding_as_client_may_send_in_one_format_and_accept_in_another() {
        let mut server = Server::new(0);
        server.start(|| { Ok(EchoBodyHandler {}) });

        let mut client = Client::new("127.0.0.1", server.port, None);
        let headers = Headers::from(vec!(
            ("Transfer-Encoding", "gzip, chunked"),
            ("Content-Encoding", "br"),
        ));

        let str = "Some body";
        let request = Request::post(
            Uri::parse("/"),
            headers,
            BodyStream(Box::new(str.as_bytes())));

        client.handle(request, |res| {
            assert_eq!(str, body_string(res.body));
            assert_eq!(res.status, OK);
            assert_eq!(res.headers.vec, vec!(
                ("Transfer-Encoding".to_string(), "gzip, chunked".to_string()),
            ));
        })
    }

    #[allow(non_snake_case)]
    #[test]
    fn TE_wins_over_content_encoding_as_client_may_send_in_one_format_and_accept_in_another() {
        let mut server = Server::new(0);
        server.start(|| { Ok(EchoBodyHandler {}) });

        let mut client = Client::new("127.0.0.1", server.port, None);
        let headers = Headers::from(vec!(
            ("Transfer-Encoding", "chunked"),
            ("TE", "trailers, deflate;q=0.5, brotli;q=0.1"),
            ("Content-Encoding", "br"),
        ));

        let str = "Some body";
        let request = Request::post(
            Uri::parse("/"),
            headers,
            BodyStream(Box::new(str.as_bytes())));

        client.handle(request, |res| {
            assert_eq!(str, body_string(res.body));
            assert_eq!(res.status, OK);
            assert_eq!(res.headers.vec, vec!(
                ("Transfer-Encoding".to_string(), "deflate, chunked".to_string()),
            ));
        })
    }


    #[allow(non_snake_case)]
    #[test]
    fn if_content_encoding_on_its_way_in_then_dont_compress_on_way_back_without_accept_encoding() {
        let mut server = Server::new(0);
        server.start(|| { Ok(EchoBodyHandler {}) });

        let mut client = Client::new("127.0.0.1", server.port, None);
        let headers = Headers::from(vec!(
            ("Content-Encoding", "br"),
        ));

        let str = "Some body";
        let request = Request::post(
            Uri::parse("/"),
            headers,
            BodyString(str));

        client.handle(request, |res| {
            assert_eq!(str, body_string(res.body));
            assert_eq!(res.status, OK);
            assert_eq!(res.headers.vec, vec!(
                ("Content-Length".to_string(), "9".to_string()),
            ));
        })
    }
}