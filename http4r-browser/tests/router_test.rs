#[cfg(test)]
mod tests {
    use http4r_browser::router::NicerRouter;
    use http4r_core::handler::Handler;
    use http4r_core::headers::Headers;
    use http4r_core::uri::Uri;
    use http4r_core::http_message::{Body, body_string, Request};
    use http4r_core::http_message::Body::BodyString;
    use http4r_core::http_message::Status::OK;

    #[test]
    fn nice_routing() {
        let mut router = NicerRouter {};

        router.handle(Request::get(Uri::parse("/some/path"), Headers::empty()), |res| {
            assert_eq!(res.status, OK);
            assert_eq!(body_string(res.body), "GET");
        });

        router.handle(Request::post(Uri::parse("/some/path"), Headers::empty(), Body::empty()), |res| {
            assert_eq!(res.status, OK);
            assert_eq!(body_string(res.body), "POST");
        })

    }
}