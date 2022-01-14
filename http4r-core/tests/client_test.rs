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

A process for decoding the chunked transfer coding can be represented
   in pseudo-code as:

     length := 0
     read chunk-size, chunk-ext (if any), and CRLF
     while (chunk-size > 0) {
        read chunk-data and CRLF
        append chunk-data to decoded-body
        length := length + chunk-size
        read chunk-size, chunk-ext (if any), and CRLF
     }
     read trailer field
     while (trailer field is not empty) {
        if (trailer field is allowed to be sent in a trailer) {
            append trailer field to existing header fields
        }
        read trailer-field
     }
     Content-Length := length
     Remove "chunked" from Transfer-Encoding
     Remove Trailer from existing header fields

 */


