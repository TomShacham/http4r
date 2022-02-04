mod common;

#[cfg(test)]
mod tests {
    use http4r_core::client::{Client};
    use http4r_core::handler::Handler;
    use http4r_core::headers::Headers;
    use http4r_core::http_message::{body_string, Request};
    use http4r_core::http_message::Body::BodyString;
    use http4r_core::server::Server;
    use http4r_core::uri::Uri;

    use crate::common::{PassHeadersAsBody};

    // test that setting TE header will set the Connection: TE header also
    #[allow(non_snake_case)]
    #[test]
    fn setting_a_TE_header_will_ensure_it_is_set_in_Connection_header_also(){
        let mut server = Server::new(0);
        server.test(|| { Ok(PassHeadersAsBody {}) });

        let mut client = Client::new("127.0.0.1", server.port, None);

        // no Connection header
        let headers = Headers::from(vec!(("TE", "trailers"), ("Transfer-Encoding", "chunked")));
        client.handle(Request::post(Uri::parse("/"), headers, BodyString("Some body")), |res| {
            assert_eq!(body_string(res.body), "TE: trailers\r\nTransfer-Encoding: chunked\r\nConnection: TE")
        });

        // already set connection header but without TE
        let headers = Headers::from(vec!(("TE", "trailers"), ("Transfer-Encoding", "chunked"), ("Connection", "close")));
        client.handle(Request::post(Uri::parse("/"), headers, BodyString("Some body")), |res| {
            assert_eq!(body_string(res.body), "TE: trailers\r\nTransfer-Encoding: chunked\r\nConnection: close, TE")
        });

        // set connection header with TE keeps it intact
        let headers = Headers::from(vec!(("TE", "trailers"), ("Transfer-Encoding", "chunked"), ("Connection", "TE")));
        client.handle(Request::post(Uri::parse("/"), headers, BodyString("Some body")), |res| {
            assert_eq!(body_string(res.body), "TE: trailers\r\nTransfer-Encoding: chunked\r\nConnection: TE")
        })
    }

    //todo() test that the client will do a chunked transfer encoding if we dont have content length
    // and we have a bodystream (ie we cant know content length ahead of time)

}

/*

todo()
A sender MUST NOT send a Content-Length header field in any message
   that contains a Transfer-Encoding header field.

A user agent SHOULD send a Content-Length in a request message when
   no Transfer-Encoding is sent and the request method defines a meaning
   for an enclosed payload body.  For example, a Content-Length header
   field is normally sent in a POST request even when the value is 0
   (indicating an empty payload body).  A user agent SHOULD NOT send a
   Content-Length header field when the request message does not contain
   a payload body and the method semantics do not anticipate such a
   body.

 */


