use http4r_core::handler::Handler;
use http4r_core::headers::Headers;
use http4r_core::http_message::{Request, Response};
use http4r_core::http_message::Body::BodyString;
use http4r_core::uri::Uri;

#[cfg(test)]
mod tests {
    use http4r_core::client::{Client};
    use http4r_core::handler::Handler;
    use http4r_core::headers::Headers;
    use http4r_core::http_message::{body_string, Request};
    use http4r_core::http_message::Status::NotFound;
    use http4r_core::server::Server;
    use http4r_core::uri::Uri;
    use http4r_example_app::app::App;

    #[test]
    fn home_page() {
        let mut server = Server::new(0);
        server.test(|| {
            let app = App::new();
            Ok(app) });

        let mut client = Client::new("127.0.0.1", server.port, None);
        client.handle(Request::get(Uri::parse("/"), Headers::empty()), |res| {
            assert_eq!(body_string(res.body), "Not found.");
        })
    }
}