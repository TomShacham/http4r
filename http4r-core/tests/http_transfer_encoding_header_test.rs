mod common;

#[cfg(test)]
mod tests {
    use http4r_core::client::Client;
    use http4r_core::handler::Handler;
    use http4r_core::headers::Headers;
    use http4r_core::http_message::Body::BodyString;
    use http4r_core::http_message::{body_string, Request, Response};
    use http4r_core::http_message::Status::OK;
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

    //test getting n chunks and having to carry on parsing half way through a chunk etc

    //test that we remove transfer encoding unless trailer set to accept it ?

    //test if client sets both content length and transfer encoding then delete transfer encoding?


}
