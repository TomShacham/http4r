mod common;

#[cfg(test)]
mod tests {
    use http4r_core::client::Client;
    use http4r_core::handler::Handler;
    use http4r_core::headers::Headers;
    use http4r_core::http_message::{body_string, Request, Response};
    use http4r_core::http_message::Body::BodyString;
    use http4r_core::http_message::Status::OK;
    use http4r_core::logging_handler::{LoggingHttpHandler, RustLogger, WasmClock};
    use http4r_core::server::Server;
    use http4r_core::uri::Uri;

    use crate::common::PassThroughHandler;

    /*
        If a message is received with both a Transfer-Encoding and a
           Content-Length header field, the Transfer-Encoding overrides the
           Content-Length.  Such a message might indicate an attempt to
           perform request smuggling (Section 9.5) or response splitting
           (Section 9.4) and ought to be handled as an error.  A sender MUST
           remove the received Content-Length field prior to forwarding such
           a message downstream.
         */
    #[test]
    fn transfer_encoding_wins_over_content_length() {
        let mut server = Server::new(0);
        server.test(|| { Ok(PassThroughHandler {}) });

        let mut client = Client { base_uri: String::from("127.0.0.1"), port: server.port };

        let little_string = "hello my baby, hello my honey, hello my ragtime gal";

        let big_chunked_request = Request::post(
            Uri::parse("/bob"),
            Headers::from(vec!(("Transfer-Encoding", "chunked"))),
            BodyString(little_string));

        client.handle(big_chunked_request, |response: Response| {
            assert_eq!(OK, response.status);
            assert_eq!(little_string, body_string(response.body));
            // Transfer-Encoding header should NOT be here now
            assert_eq!(vec!(("Content-Length".to_string(), "51".to_string())), response.headers.vec);
        });
    }

    #[test]
    fn large_chunked_request() {
        let mut server = Server::new(0);
        server.test(|| { Ok(PassThroughHandler {}) });

        let mut client = Client { base_uri: String::from("127.0.0.1"), port: server.port };

        let long_string = "hello my baby hello my honey, hello my ragtime gal! ".repeat(1000);

        let big_chunked_request = Request::post(
            Uri::parse("/bob"),
            Headers::from(vec!(("Transfer-Encoding", "chunked"))),
            BodyString(long_string.as_str()));

        client.handle(big_chunked_request, |response: Response| {
            assert_eq!(OK, response.status);
            assert_eq!(long_string, body_string(response.body));
            // Transfer-Encoding header should NOT be here now
            assert_eq!(vec!(("Content-Length".to_string(), "52000".to_string())), response.headers.vec);
        });
    }

    #[test]
    fn can_include_boundary_characters_in_chunk() {
        let mut server = Server::new(0);
        server.test(|| { Ok(PassThroughHandler {}) });

        let mut client = Client { base_uri: String::from("127.0.0.1"), port: server.port };
        let with_encoding = "hello\r\n";

        let big_chunked_request = Request::post(
            Uri::parse("/bob"),
            Headers::from(vec!(("Transfer-Encoding", "chunked"))),
            BodyString(with_encoding));

        client.handle(big_chunked_request, |response: Response| {
            assert_eq!(OK, response.status);
            assert_eq!(with_encoding, body_string(response.body));
            // Transfer-Encoding header should NOT be here now
            assert_eq!(vec!(("Content-Length".to_string(), "7".to_string())), response.headers.vec);
        });
    }

    /*
        When a message includes a message body encoded with the chunked
       transfer coding and the sender desires to send metadata in the form
       of trailer fields at the end of the message, the sender SHOULD
       generate a Trailer header field before the message body to indicate
       which fields will be present in the trailers.  This allows the
       recipient to prepare for receipt of that metadata before it starts
       processing the body, which is useful if the message is being streamed
       and the recipient wishes to confirm an integrity check on the fly.

         Trailer = 1#field-name
     */
    #[test]
    fn supports_trailers_parsing_into_a_body() {
        let mut server = Server::new(0);
        server.test(|| { Ok(PassThroughHandler {}) });

        let mut client = Client { base_uri: "127.0.0.1".to_string(), port: server.port };

        let little_string = "hello";

        let with_trailer = Request::post(
            Uri::parse("/bob"),
            Headers::from(vec!(
                ("Transfer-Encoding", "chunked"),
                ("Trailer", "Expires, Integrity"),
            )),
            BodyString(little_string),
        ).with_trailers(Headers::from(vec!(
            ("Expires", "Wed, 21 Oct 2015 07:28:00 GMT"),
            ("Integrity", "Some hash"),
        )));

        client.handle(with_trailer, |response: Response| {
            assert_eq!(OK, response.status);
            assert_eq!(little_string, body_string(response.body));
            // Transfer-Encoding header should NOT be here now
            assert_eq!(vec!(
                ("Expires".to_string(), "Wed, 21 Oct 2015 07:28:00 GMT".to_string()),
                ("Integrity".to_string(), "Some hash".to_string()),
                ("Trailer".to_string(), "Expires, Integrity".to_string()),
                ("Content-Length".to_string(), "5".to_string()),
            ), response.headers.vec);
        });
    }

    /*
https://datatracker.ietf.org/doc/html/rfc7230#section-4.1.2

A sender MUST NOT generate a trailer that contains a field necessary
   for message framing (e.g., Transfer-Encoding and Content-Length),
   routing (e.g., Host), request modifiers (e.g., controls and
   conditionals in Section 5 of [RFC7231]), authentication (e.g., see
   [RFC7235] and [RFC6265]), response control data (e.g., see Section
   7.1 of [RFC7231]), or determining how to process the payload (e.g.,
   Content-Encoding, Content-Type, Content-Range, and Trailer).

When a chunked message containing a non-empty trailer is received,
   the recipient MAY process the fields (aside from those forbidden
   above) as if they were appended to the message's header section.  A
   recipient MUST ignore (or consider as an error) any fields that are
   forbidden to be sent in a trailer, since processing them as if they
   were present in the header section might bypass external security
   filters.

   Unless the request includes a TE header field indicating "trailers"
   is acceptable, as described in Section 4.3, a server SHOULD NOT
   generate trailer fields that it believes are necessary for the user
   agent to receive.  Without a TE containing "trailers", the server
   ought to assume that the trailer fields might be silently discarded
   along the path to the user agent.  This requirement allows
   intermediaries to forward a de-chunked message to an HTTP/1.0
   recipient without buffering the entire response.

Transfer-Encoding and Content-Length
Host
Cache-Control, Max-Forwards, or TE
 Authorization or Set-Cookie
Content-Encoding, Content-Type, Content-Range

*/
    #[test]
    fn cannot_set_certain_trailers_as_headers() {
        let mut server = Server::new(0);
        server.test(|| { Ok(PassThroughHandler {}) });

        let mut client = Client { base_uri: "127.0.0.1".to_string(), port: server.port };

        let little_string = "hello";

        let with_illegal_trailers = Request::post(
            Uri::parse("/bob"),
            Headers::from(vec!(
                ("Transfer-Encoding", "chunked"),
                ("Trailer", "Expires, Transfer-Encoding, Content-Length, Cache-Control, Max-Forwards, TE, Authorization, Set-Cookie, Content-Encoding, Content-Type, Content-Range")
            )),
            BodyString(little_string),
        ).with_trailers(Headers::from(vec!(
            ("Expires", "Wed, 21 Oct 2015 07:28:00 GMT"),
            ("Transfer-Encoding", "chunked"),
            ("Content-Length", "10"),
            ("Cache-Control", "private"),
            ("Max-Forwards", "50"),
            ("TE", "trailers"),
            ("Trailer", "Content-Length"),
            ("Authorization", "foo@bar"),
            ("Set-Cookie", "tom=foo; matt=bar"),
            ("Content-Encoding", "gzip"),
            ("Content-Type", "text/html"),
            ("Content-Range", "bytes 200-1000/67589"),
            //todo() add full list of disallowed trailers       https://datatracker.ietf.org/doc/html/rfc7230#section-4.1.2
        )));

        client.handle(with_illegal_trailers, |response: Response| {
            assert_eq!(OK, response.status);
            assert_eq!(little_string, body_string(response.body));
            // Transfer-Encoding header should NOT be here now
            assert_eq!(vec!(
                ("Expires".to_string(), "Wed, 21 Oct 2015 07:28:00 GMT".to_string()),
                ("Trailer".to_string(), "Expires, Transfer-Encoding, Content-Length, Cache-Control, Max-Forwards, TE, Authorization, Set-Cookie, Content-Encoding, Content-Type, Content-Range".to_string()),
                ("Content-Length".to_string(), "5".to_string()),
            ), response.headers.vec);
        });
    }

    //test trailers cant be really long

    //test that we remove transfer encoding unless trailer set to accept it ?

    //test that if client accepts trailers TE then it keeps em and we keep message chunked ???
}
