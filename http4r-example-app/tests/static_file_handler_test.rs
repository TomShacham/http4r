mod common;

#[cfg(test)]
mod tests {
    use std::fs::{canonicalize};
    use std::io::{Read};

    use http4r_core::client::Client;
    use http4r_core::handler::Handler;
    use http4r_core::headers::Headers;
    use http4r_core::http_message::{Body, Request};
    use http4r_core::http_message::Status::{OK};
    use http4r_core::server::Server;
    use http4r_core::uri::Uri;

    use http4r_example_app::static_file_handler::StaticFileHandler;


    #[test]
    fn canonicalizing_paths() {
        assert_eq!(canonicalize(".").unwrap().to_str().unwrap(), "/Users/tom/Projects/http4r/http4r-example-app");
        assert_eq!(canonicalize("./resources/html").unwrap().to_str().unwrap(), "/Users/tom/Projects/http4r/http4r-example-app/resources/html")
    }

    #[test]
    fn static_file_handler_loads_icons() {
        let mut static_file_handler = StaticFileHandler::new("/resources/html", "test".to_string());

        static_file_handler.handle(Request::get(Uri::parse("/favicon.ico"), Headers::empty()), |response| {
            assert_eq!(response.status, OK);
            match response.body {
                Body::BodyString(_) => panic!("body string for image"),
                Body::BodyStream(mut read) => {
                    let mut vec: Vec<u8> = Vec::new();
                    read.read_to_end(&mut vec).unwrap();
                    assert_eq!(vec.len(), 3758);
                }
            }
        })
    }

    #[test]
    fn static_file_handler_loads_icons_over_http() {
        let mut server = Server::new(0);

        server.start(|| { Ok(StaticFileHandler::new("/resources/html", "test".to_string()))}, true);

        let mut client = Client::new("127.0.0.1", server.port, None);

        client.handle(Request::get(Uri::parse("/favicon.ico"), Headers::empty()), |response| {
            assert_eq!(response.status, OK);
            match response.body {
                Body::BodyString(_) => panic!("body string for image"),
                Body::BodyStream(mut read) => {
                    let mut vec: Vec<u8> = Vec::new();
                    read.read_to_end(&mut vec).unwrap();
                    assert_eq!(vec.len(), 3758);
                }
            }
        })
    }
}

