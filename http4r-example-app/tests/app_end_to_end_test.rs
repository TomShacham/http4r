use http4r_core::handler::Handler;
use http4r_core::headers::Headers;
use http4r_core::http_message::{Request, Response};
use http4r_core::http_message::Body::BodyString;
use http4r_core::uri::Uri;

mod common;

#[cfg(test)]
mod tests {
    use std::fs;
    use std::fs::File;
    use std::io::{Read, Write};
    use std::str::{from_utf8, Utf8Error};
    use http4r_core::client::{Client};
    use http4r_core::handler::Handler;
    use http4r_core::headers::Headers;
    use http4r_core::http_message::{body_string, Request};
    use http4r_core::http_message::Status::NotFound;
    use http4r_core::server::Server;
    use http4r_core::uri::Uri;
    use http4r_example_app::app::App;
    use http4r_example_app::static_file_handler::StaticFileHandler;
    use crate::common::approve;

    #[test]
    fn static_file_handler() {
        let mut server = Server::new(0);
        server.test(|| { Ok(App::new(StaticFileHandler::new("./resources/html"))) });

        let mut client = Client::new("127.0.0.1", server.port, None);
        client.handle(Request::get(Uri::parse("/"), Headers::empty()), |res| {
            approve(body_string(res.body), "./resources/html/index.html");
        })
    }
}

