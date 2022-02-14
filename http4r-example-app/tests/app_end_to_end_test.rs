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
    use http4r_core::http_message::{body_string, Request};
    use http4r_core::http_message::Status::{Forbidden, NotFound};
    use http4r_core::server::Server;
    use http4r_core::uri::Uri;
    use http4r_example_app::app::App;
    use http4r_example_app::environment::Environment;
    use http4r_example_app::static_file_handler::StaticFileHandler;
    use crate::common::approve;

    #[test]
    fn static_file_handler_home_page() {
        let mut server = Server::new(0);
        server.test(|| { Ok(App::in_memory(Environment::empty())) });

        let mut client = Client::new("127.0.0.1", server.port, None);
        client.handle(Request::get(Uri::parse("/"), Headers::empty()), |res| {
            approve(body_string(res.body), "/resources/html/index.html");
        })
    }

    #[test]
    fn file_not_found_results_in_404() {
        let mut server = Server::new(0);
        server.test(|| { Ok(App::in_memory(Environment::empty())) });

        let mut client = Client::new("127.0.0.1", server.port, None);
        client.handle(Request::get(Uri::parse("/some/unknown/file.html"), Headers::empty()), |res| {
            assert_eq!(body_string(res.body), "Not found.");
        })
    }

    #[test]
    fn attempt_to_go_outside_root_causes_403_in_static_file_handler_and_is_turned_into_404_to_be_opaque_to_user_agent() {
        let mut handler = StaticFileHandler::new("/resources/html", "test".to_string());
        let request_going_outside_root = Request::get(Uri::parse("/../not-allowed-to-view.html"), Headers::empty());
        handler.handle(request_going_outside_root, |res| {
            assert_eq!(body_string(res.body), format!("Attempted to access a file outside of root: {}/resources/not-allowed-to-view.html", env::current_dir().unwrap().to_str().unwrap()));
            assert_eq!(res.status, Forbidden);
        });

        let request_going_outside_root = Request::get(Uri::parse("/../not-allowed-to-view.html"), Headers::empty());
        let mut app = App::in_memory(Environment::empty());
        app.handle(request_going_outside_root, |res| {
            approve(body_string(res.body), "./resources/html/not-found.html");
        })
    }
}

