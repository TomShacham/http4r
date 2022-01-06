mod common;

#[cfg(test)]
mod tests {
    use http4r_core::client::Client;
    use http4r_core::handler::Handler;
    use http4r_core::headers::Headers;
    use http4r_core::http_message::Body::BodyString;
    use http4r_core::http_message::{body_string, Request, Response};
    use http4r_core::http_message::Status::{BadRequest, OK};
    use http4r_core::server::Server;
    use http4r_core::uri::Uri;
    use crate::common::PassThroughHandler;


    #[test]
    fn duplicate_different_content_length_headers_result_in_bad_request() {
        let mut server = Server::new(0);
        server.test(|| { Ok(PassThroughHandler {}) });

        let mut client = Client { base_uri: String::from("127.0.0.1"), port: server.port };

        // multiple content-lengths of different value
        client.handle(Request::post(Uri::parse("/bob"), Headers::from(
            vec!(("Content-length", "5"), ("Content-length", "10"))
        ), BodyString("hello")), |response: Response| {
            assert_eq!(BadRequest, response.status);
            assert_eq!("Content Length header couldn't be parsed, got 5, 10", body_string(response.body));
        });
    }

    #[test]
    fn duplicate_same_content_lengths_are_fine(){
        let mut server = Server::new(0);
        server.test(|| { Ok(PassThroughHandler {}) });

        let mut client = Client { base_uri: String::from("127.0.0.1"), port: server.port };

        // content-length is duplicated but the same
        client.handle(Request::post(Uri::parse("/bob"), Headers::from(
            vec!(("Content-length", "5"), ("Content-length", "5"))
        ), BodyString("hello")), |response: Response| {
            assert_eq!(OK, response.status);
            assert_eq!("hello", body_string(response.body));
        });
    }

    #[test]
    fn content_length_must_be_positive(){
        let mut server = Server::new(0);
        server.test(|| { Ok(PassThroughHandler {}) });

        let mut client = Client { base_uri: String::from("127.0.0.1"), port: server.port };

        // content-length is negative
        client.handle(Request::post(Uri::parse("/bob"), Headers::from(
            vec!(("Content-length", "-5"))
        ), BodyString("hello")), |response: Response| {
            assert_eq!(BadRequest, response.status);
            assert_eq!("Content Length header couldn't be parsed, got -5", body_string(response.body));
        });
    }

    #[test]
    fn duplicate_invalid_lengths_are_invalid() {
        let mut server = Server::new(0);
        server.test(|| { Ok(PassThroughHandler {}) });

        let mut client = Client { base_uri: String::from("127.0.0.1"), port: server.port };

        // content-length is duplicated and invalid
        client.handle(Request::post(Uri::parse("/bob"), Headers::from(
            vec!(("Content-length", "-5"), ("Content-length", "-5"))
        ), BodyString("hello")), |response: Response| {
            assert_eq!(BadRequest, response.status);
            assert_eq!("Content Length header couldn't be parsed, got -5", body_string(response.body));
        });
    }
}

/*

https://datatracker.ietf.org/doc/html/rfc7230#section-3.3

If a message is received that has multiple Content-Length header
   fields with field-values consisting of the same decimal value, or a
   single Content-Length header field with a field value containing a
   list of identical decimal values (e.g., "Content-Length: 42, 42"),
   indicating that duplicate Content-Length header fields have been
   generated or combined by an upstream message processor, then the
   recipient MUST either reject the message as invalid or replace the
   duplicated field-values with a single valid Content-Length field
   containing that decimal value prior to determining the message body
   length or forwarding the message.

      Note: HTTP's use of Content-Length for message framing differs
      significantly from the same field's use in MIME, where it is an
      optional field used only within the "message/external-body"
      media-type.
 */
