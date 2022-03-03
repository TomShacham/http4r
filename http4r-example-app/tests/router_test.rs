#[cfg(test)]
mod tests {
    use http4r_core::handler::Handler;
    use http4r_core::headers::Headers;
    use http4r_core::http_message::{body_string, Request};
    use http4r_core::http_message::Status::{OK};
    use http4r_core::uri::Uri;

    use http4r_example_app::router::Router;

    #[test]
    fn router() {
        let request = Request::get(
            Uri::parse("/site/bob/profile?org=uncles"),
            Headers::from(vec!(("son", "jim"), ("daughter", "jam"))));
        let mut router = Router::new();
        // router is also a Handler so we can just handle a request directly
        // "in-memory" rather than going over HTTP
        router.handle(request, |res| {
            assert_eq!(res.status, OK);
            assert_eq!(body_string(res.body), "Uncle->Bob: Jam, Jim".to_string())
        });
    }
}