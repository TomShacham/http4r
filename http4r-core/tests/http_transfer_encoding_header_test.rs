mod common;

#[cfg(test)]
mod tests {
    use std::io::{Cursor, Write};
    use std::net::TcpStream;
    use http4r_core::client::Client;
    use http4r_core::handler::Handler;
    use http4r_core::headers::{Headers, HeaderType};
    use http4r_core::http_message::{body_string, Request, Response, Status};
    use http4r_core::http_message::Body::{BodyStream, BodyString};
    use http4r_core::http_message::Status::{BadRequest, OK};
    use http4r_core::server::Server;
    use http4r_core::uri::Uri;

    use crate::common::{MalformedChunkedEncodingClient, PassThroughHandler};

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

        let mut client = Client::new("127.0.0.1", server.port, None);

        let little_string = "hello my baby, hello my honey, hello my ragtime gal";

        let big_chunked_request = Request::post(
            Uri::parse("/bob"),
            Headers::from(vec!(("Transfer-Encoding", "chunked"))),
            BodyString(little_string));

        client.handle(big_chunked_request, |response: Response| {
            assert_eq!(little_string, body_string(response.body));
            assert_eq!(OK, response.status);
            assert_eq!(vec!(("Transfer-Encoding".to_string(), "chunked".to_string())), response.headers.vec);
        });
    }

    #[test]
    fn large_chunked_request() {
        let mut server = Server::new(0);
        server.test(|| { Ok(PassThroughHandler {}) });

        let mut client = Client::new("127.0.0.1", server.port, None);

        let long_string = "hello my baby hello my honey, hello my ragtime gal! ".repeat(1000);

        let big_chunked_request = Request::post(
            Uri::parse("/bob"),
            Headers::from(vec!(("Transfer-Encoding", "chunked"))),
            BodyString(long_string.as_str()));

        client.handle(big_chunked_request, |response: Response| {
            assert_eq!(long_string, body_string(response.body));
            assert_eq!(OK, response.status);
            assert_eq!(vec!(("Transfer-Encoding".to_string(), "chunked".to_string())), response.headers.vec);
        });
    }

    #[test]
    fn can_include_boundary_characters_in_chunk() {
        let mut server = Server::new(0);
        server.test(|| { Ok(PassThroughHandler {}) });

        let mut client = Client::new("127.0.0.1", server.port, None);
        let with_encoding = "hello\r\n";

        let big_chunked_request = Request::post(
            Uri::parse("/bob"),
            Headers::from(vec!(("Transfer-Encoding", "chunked"))),
            BodyString(with_encoding));

        client.handle(big_chunked_request, |response: Response| {
            assert_eq!(OK, response.status);
            assert_eq!(with_encoding, body_string(response.body));
            assert_eq!(vec!(("Transfer-Encoding".to_string(), "chunked".to_string())), response.headers.vec);
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

        let mut client = Client::new("127.0.0.1", server.port, None);

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
            assert_eq!(vec!(
                ("Expires".to_string(), "Wed, 21 Oct 2015 07:28:00 GMT".to_string()),
                ("Integrity".to_string(), "Some hash".to_string()),
                ("Transfer-Encoding".to_string(), "chunked".to_string()),
                ("Trailer".to_string(), "Expires, Integrity".to_string()),
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
*/
    #[test]
    fn cannot_set_certain_trailers_as_headers() {
        let mut server = Server::new(0);
        server.test(|| { Ok(PassThroughHandler {}) });

        let mut client = Client::new("127.0.0.1", server.port, None);

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
        )));

        client.handle(with_illegal_trailers, |response: Response| {
            assert_eq!(OK, response.status);
            assert_eq!(little_string, body_string(response.body));
            assert!(response.trailers.vec.is_empty()); // trailers are empty
            assert_eq!(vec!(
                ("Expires".to_string(), "Wed, 21 Oct 2015 07:28:00 GMT".to_string()), // valid trailer gets added to header
                ("Transfer-Encoding".to_string(), "chunked".to_string()),
                ("Trailer".to_string(), "Expires, Transfer-Encoding, Content-Length, Cache-Control, Max-Forwards, TE, Authorization, Set-Cookie, Content-Encoding, Content-Type, Content-Range".to_string()),
            ), response.headers.vec);
        });
    }

    /*
       Unless the request includes a TE header field indicating "trailers"
       is acceptable, as described in Section 4.3, a server SHOULD NOT
       generate trailer fields that it believes are necessary for the user
       agent to receive.  Without a TE containing "trailers", the server
       ought to assume that the trailer fields might be silently discarded
       along the path to the user agent.  This requirement allows
       intermediaries to forward a de-chunked message to an HTTP/1.0
       recipient without buffering the entire response.
     */
    #[allow(non_snake_case)]
    #[test]
    fn dont_get_trailers_unless_TE_specified() {
        let mut server = Server::new(0);
        server.test(|| { Ok(PassThroughHandler {}) });
        let mut client = Client::new("127.0.0.1", server.port, None);

        let body = "hello";
        let do_not_want_trailers = Request::post(
            Uri::parse("/bob"),
            Headers::from(vec!(
                ("Transfer-Encoding", "chunked"),
                ("Trailer", "Expires"),
                // => no TE header
            )),
            BodyString(body),
        ).with_trailers(Headers::from(vec!(
            ("Expires", "Wed, 21 Oct 2015 07:28:00 GMT"),
        )));

        client.handle(do_not_want_trailers, |response: Response| {
            assert_eq!(OK, response.status);
            assert_eq!(body, body_string(response.body));
            assert_eq!(vec!(
                //Expires is in headers not trailers now
                ("Expires".to_string(), "Wed, 21 Oct 2015 07:28:00 GMT".to_string()),
                ("Transfer-Encoding".to_string(), "chunked".to_string()),
                ("Trailer".to_string(), "Expires".to_string()),
            ), response.headers.vec);
            let vec1: Vec<HeaderType> = vec!(
                // => no Expires trailer in trailers
            );
            assert_eq!(vec1, response.trailers.vec);
        });

        let asks_for_trailers = Request::post(
            Uri::parse("/bob"),
            Headers::from(vec!(
                ("Transfer-Encoding", "chunked"),
                ("Trailer", "Expires"),
                ("TE", "trailers")
            )),
            BodyString(body),
        ).with_trailers(Headers::from(vec!(
            ("Expires", "Wed, 21 Oct 2015 07:28:00 GMT"),
        )));

        client.handle(asks_for_trailers, |response: Response| {
            assert_eq!(OK, response.status);
            assert_eq!(body, body_string(response.body));
            assert_eq!(vec!(
                //Expires should be in trailers
                ("Transfer-Encoding".to_string(), "chunked".to_string()),
                ("Trailer".to_string(), "Expires".to_string()),
                // TE trailer should be removed
            ), response.headers.vec);
            assert_eq!(vec!(
                ("Expires".to_string(), "Wed, 21 Oct 2015 07:28:00 GMT".to_string()),
            ), response.trailers.vec);
        });
    }


    /*
    The "TE" header field in a request indicates what transfer codings,
   besides chunked, the client is willing to accept in response, and
   whether or not the client is willing to accept trailer fields in a
   chunked transfer coding.

   The TE field-value consists of a comma-separated list of transfer
   coding names, each allowing for optional parameters (as described in
   Section 4), and/or the keyword "trailers".  A client MUST NOT send
   the chunked transfer coding name in TE; chunked is always acceptable
   for HTTP/1.1 recipients.

     TE        = #t-codings
     t-codings = "trailers" / ( transfer-coding [ t-ranking ] )
     t-ranking = OWS ";" OWS "q=" rank
     rank      = ( "0" [ "." 0*3DIGIT ] )
                / ( "1" [ "." 0*3("0") ] )

   Three examples of TE use are below.

     TE: deflate
     TE:
     TE: trailers, deflate;q=0.5

   The presence of the keyword "trailers" indicates that the client is
   willing to accept trailer fields in a chunked transfer coding, as
   defined in Section 4.1.2, on behalf of itself and any downstream
   clients.  For requests from an intermediary, this implies that
   either: (a) all downstream clients are willing to accept trailer
   fields in the forwarded response; or, (b) the intermediary will
   attempt to buffer the response on behalf of downstream recipients.
   Note that HTTP/1.1 does not define any means to limit the size of a
   chunked response such that an intermediary can be assured of
   buffering the entire response.

   When multiple transfer codings are acceptable, the client MAY rank
   the codings by preference using a case-insensitive "q" parameter
   (similar to the qvalues used in content negotiation fields, Section
   5.3.1 of [RFC7231]).  The rank value is a real number in the range 0
   through 1, where 0.001 is the least preferred and 1 is the most
   preferred; a value of 0 means "not acceptable".

   If the TE field-value is empty or if no TE field is present, the only
   acceptable transfer coding is chunked.  A message with no transfer
   coding is always acceptable.

   Since the TE header field only applies to the immediate connection, a
   sender of TE MUST also send a "TE" connection option within the
   Connection header field (Section 6.1) in order to prevent the TE
   field from being forwarded by intermediaries that do not support its
   semantics.
     */

    #[allow(non_snake_case)]
    #[test]
    fn TE_with_ranked_compression() {
        let mut server = Server::new(0);
        server.test(|| { Ok(PassThroughHandler {}) });
        let mut client = Client::new("127.0.0.1", server.port, None);

        let body = "hello".repeat(10000);
        let chunked_with_TE_gzip = Request::post(
            Uri::parse("/bob"),
            Headers::from(vec!(
                ("Transfer-Encoding", "chunked"),
                ("Trailer", "Expires"),
                ("TE", "trailers, deflate;q=0.5, gzip;q=0.9"),
                ("Connection", "TE"),
            )),
            BodyString(body.as_str()),
        ).with_trailers(Headers::from(vec!(
            ("Expires", "Wed, 21 Oct 2015 07:28:00 GMT")
        )));

        client.handle(chunked_with_TE_gzip, |response: Response| {
            assert_eq!(body_string(response.body), body);
            assert_eq!(OK, response.status);
            assert_eq!(vec!(
                ("Transfer-Encoding".to_string(), "gzip, chunked".to_string()),
                ("Trailer".to_string(), "Expires".to_string()),
            ), response.headers.vec);
            assert_eq!(vec!(("Expires".to_string(), "Wed, 21 Oct 2015 07:28:00 GMT".to_string())),
                       response.trailers.vec);
        });

        let chunked_with_TE_deflate = Request::post(
            Uri::parse("/bob"),
            Headers::from(vec!(
                ("Transfer-Encoding", "chunked"),
                ("Trailer", "Expires"),
                ("TE", "trailers, gzip;q=0.1, deflate;q=0.9"),
                ("Connection", "TE"),
            )),
            BodyString(body.as_str()),
        ).with_trailers(Headers::from(vec!(
            ("Expires", "Wed, 21 Oct 2015 07:28:00 GMT")
        )));

        client.handle(chunked_with_TE_deflate, |response: Response| {
            assert_eq!(body_string(response.body), body);
            assert_eq!(OK, response.status);
            assert_eq!(vec!(
                ("Transfer-Encoding".to_string(), "deflate, chunked".to_string()),
                ("Trailer".to_string(), "Expires".to_string()),
            ), response.headers.vec);
            assert_eq!(vec!(("Expires".to_string(), "Wed, 21 Oct 2015 07:28:00 GMT".to_string())),
                       response.trailers.vec);
        });

        let chunked_with_TE_brotli = Request::post(
            Uri::parse("/bob"),
            Headers::from(vec!(
                ("Transfer-Encoding", "chunked"),
                ("Trailer", "Expires"),
                ("TE", "trailers, gzip;q=0.1, deflate;q=0.7, brotli;q=0.9"),
                ("Connection", "TE"),
            )),
            BodyString(body.as_str()),
        ).with_trailers(Headers::from(vec!(
            ("Expires", "Wed, 21 Oct 2015 07:28:00 GMT")
        )));

        client.handle(chunked_with_TE_brotli, |response: Response| {
            assert_eq!(body_string(response.body), body);
            assert_eq!(OK, response.status);
            assert_eq!(vec!(
                ("Transfer-Encoding".to_string(), "brotli, chunked".to_string()),
                ("Trailer".to_string(), "Expires".to_string()),
                // TE header does not get echoed back in response
            ), response.headers.vec);
            assert_eq!(vec!(("Expires".to_string(), "Wed, 21 Oct 2015 07:28:00 GMT".to_string())),
                       response.trailers.vec);
        });
    }

    #[test]
    fn if_trailers_are_too_long_we_get_an_error() {
        let mut server = Server::new(0);
        server.test(|| { Ok(PassThroughHandler {}) });
        let mut client = Client::new("127.0.0.1", server.port, None);

        let very_long_trailer = "A very long trailer. ".repeat(1000);
        let body = "hello";
        let asks_for_trailers = Request::post(
            Uri::parse("/bob"),
            Headers::from(vec!(
                ("Transfer-Encoding", "chunked"),
                ("Trailer", "Expires"),
                ("TE", "trailers, deflate;q=0.5")
            )),
            BodyString(body),
        ).with_trailers(Headers::from(vec!(
            ("Expires", very_long_trailer.as_str()),
        )));

        client.handle(asks_for_trailers, |response: Response| {
            assert_eq!("Trailers must be less than 16384", body_string(response.body));
            assert_eq!(BadRequest, response.status);
            assert_eq!(vec!(
                ("Content-Length".to_string(), "32".to_string()),
            ), response.headers.vec);
            let vec1: Vec<HeaderType> = vec!();
            assert_eq!(vec1, response.trailers.vec);
        });
    }

    #[test]
    fn best_request_if_invalid_boundary_digit() {
        let mut server = Server::new(0);
        server.test(|| { Ok(PassThroughHandler {}) });
        let mut stream = TcpStream::connect(format!("127.0.0.1:{}", server.port)).unwrap();

        let _write = stream.write("GET / HTTP/1.1\r\nTransfer-Encoding: Chunked\r\n\r\n5\r\nhello\r\nX".as_bytes());

        let mut client = MalformedChunkedEncodingClient { port: server.port };

        client.handle(Request::get(Uri::parse("/"), Headers::from(vec!(("Transfer-encoding" , "chunked")))), |res| {
            assert_eq!(res.status, Status::BadRequest);
            assert_eq!(body_string(res.body), "Could not parse boundary character X in chunked encoding".to_string())
        })
    }

    #[test]
    fn writing_a_chunked_body_stream() {
        let mut server = Server::new(0);
        server.test(|| { Ok(PassThroughHandler {}) });

        let mut client = Client::new("127.0.0.1", server.port, None);

        let string1 = "hello my baby, hello my honey, hello my ragtime gal".repeat(1000);
        let little_string = string1.as_bytes().to_vec();

        let big_chunked_request = Request::post(
            Uri::parse("/bob"),
            Headers::from(vec!(("Transfer-Encoding", "chunked"))),
            BodyStream(Box::new(Cursor::new(little_string))));

        client.handle(big_chunked_request, |response: Response| {
            assert_eq!(string1, body_string(response.body));
            assert_eq!(OK, response.status);
            assert_eq!(vec!(("Transfer-Encoding".to_string(), "chunked".to_string())), response.headers.vec);
        });
    }

    // test that we read the body into non-chunked-encoding if it's http 1.0

    // test that a proxy can undo transfer encoding but must respect content encoding
}
