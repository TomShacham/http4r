
#[cfg(test)]
mod tests {
    use http4r_core::handler::Handler;
    use http4r_core::headers::Headers;
    use http4r_core::http_message::{body_string, Request};
    use http4r_core::http_message::Status::NotFound;
    use http4r_core::server::Server;
    use http4r_core::uri::Uri;
    use http4r_example_app::app::App;

    #[test]
    fn not_found() {
        let mut app = App::new();
        let get_root = Request::get(Uri::parse("/"), Headers::empty());

        app.handle(get_root, |res| {
            assert_eq!(res.status, NotFound);
            assert_eq!(body_string(res.body), "Not found.")
        });
    }
}