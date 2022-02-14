
#[cfg(test)]
mod tests {
    use http4r_core::handler::Handler;
    use http4r_core::headers::Headers;
    use http4r_core::http_message::{body_string, Request};
    use http4r_core::http_message::Status::NotFound;
    use http4r_core::server::Server;
    use http4r_core::uri::Uri;
    use http4r_example_app::app::App;
    use http4r_example_app::not_found_handler::NotFoundHandler;
    use http4r_example_app::static_file_handler::StaticFileHandler;

    #[test]
    fn not_found() {
        let mut not_found_handler = NotFoundHandler::new(StaticFileHandler::new("./resources/html", "test".to_string()));
        let unknown_route = Request::get(Uri::parse("/unknown/route"), Headers::empty());

        not_found_handler.handle(unknown_route, |res| {
            assert_eq!(res.status, NotFound);
            assert_eq!(body_string(res.body), "Our custom 404 page!")
        });
    }
}