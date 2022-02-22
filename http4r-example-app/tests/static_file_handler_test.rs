use http4r_core::handler::Handler;
use http4r_core::headers::Headers;
use http4r_core::http_message::{Request, Response};
use http4r_core::http_message::Body::BodyString;
use http4r_core::uri::Uri;

mod common;

#[cfg(test)]
mod tests {
    use std::{env, fs};
    use std::fs::File;
    use std::io::{Read, Write};
    use std::str::{from_utf8, Utf8Error};
    use http4r_core::client::{Client};
    use http4r_core::handler::Handler;
    use http4r_core::headers::Headers;
    use http4r_core::http_message::{Body, body_length, body_string, Request};
    use http4r_core::http_message::Status::{Forbidden, NotFound, OK};
    use http4r_core::server::Server;
    use http4r_core::uri::Uri;
    use http4r_example_app::app::App;
    use http4r_example_app::environment::Environment;
    use http4r_example_app::static_file_handler::StaticFileHandler;
    use crate::common::approve;

    #[test]
    fn static_file_handler_loads_icons() {
        let mut static_file_handler = StaticFileHandler::new("/resources/html", "test".to_string());

        static_file_handler.handle(Request::get(Uri::parse("/favicon.ico"), Headers::empty()), |response| {
            assert_eq!(response.status, OK);
            match response.body {
                Body::BodyString(_) => panic!("body string for image"),
                Body::BodyStream(mut read) => {
                    let mut vec: Vec<u8> = Vec::new();
                    read.read_to_end(&mut vec);
                    assert_eq!(vec.len(), 3758);
                }
            }
        })
    }

    #[test]
    fn static_file_handler_loads_icons_over_http() {
        let mut server = Server::new(0);

        server.start(|| { Ok(StaticFileHandler::new("/resources/html", "test".to_string()))} );

        let mut client = Client::new("127.0.0.1", server.port, None);

        client.handle(Request::get(Uri::parse("/favicon.ico"), Headers::empty()), |response| {
            assert_eq!(response.status, OK);
            match response.body {
                Body::BodyString(_) => panic!("body string for image"),
                Body::BodyStream(mut read) => {
                    let mut vec: Vec<u8> = Vec::new();
                    read.read_to_end(&mut vec);
                    assert_eq!(vec.len(), 3758);
                }
            }
        })
    }
}

