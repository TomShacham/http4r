#[cfg(test)]
mod tests {
    use http4r_core::handler::Handler;
    use http4r_core::headers::Headers;
    use http4r_core::http_message::{body_string, Request};
    use http4r_core::http_message::Status::{BadRequest, OK};
    use http4r_core::uri::Uri;

    use http4r_example_app::router::{ProfileRouter, Router};

    #[test]
    fn router() {
        // a GET request with path param "Bob", query param "org" and headers
        let request = Request::get(
            Uri::parse("/site/Bob/profile?org=Uncle"),
            Headers::from(vec!(("friend", "Vic"), ("friend", "Ulrika"))));
        let mut router = Router::new(ProfileRouter);
        // router is also a Handler so we can just handle a request directly
        // "in-memory" rather than going over HTTP
        router.handle(request, |res| {
            assert_eq!(res.status, OK);
            // assimilate the info given to us in the response body
            assert_eq!(body_string(res.body), "Uncle->Bob: Vic, Ulrika".to_string())
        });
    }

    #[test]
    fn router_disallows_requests_with_no_org() {
        // a GET request with no query param "org"!
        let request = Request::get(
            Uri::parse("/site/Bob/profile"),
            Headers::from(vec!(("friend", "Vic"), ("friend", "Ulrika"))));
        let mut router = Router::new(ProfileRouter);
        router.handle(request, |res| {
            assert_eq!(res.status, BadRequest);
            assert_eq!(body_string(res.body), "Expected query parameter \"org\".".to_string())
        });
    }

    #[test]
    fn router_disallows_requests_with_no_friend() {
        // a GET request with no "friend" header!
        let request = Request::get(
            Uri::parse("/site/Bob/profile?org=Uncle"),
            Headers::empty());
        let mut router = Router::new(ProfileRouter);
        router.handle(request, |res| {
            assert_eq!(res.status, BadRequest);
            assert_eq!(body_string(res.body), "Expected header \"friend\".".to_string())
        });
    }

    #[test]
    fn show_home_page() {
        // a GET request with no "friend" header!
        let request = Request::get(Uri::parse("/home"), Headers::empty());
        let mut router = Router::new(ProfileRouter);
        router.handle(request, |res| {
            assert_eq!(res.status, OK);
            assert_eq!(body_string(res.body), "Home page.".to_string())
        });
    }
}