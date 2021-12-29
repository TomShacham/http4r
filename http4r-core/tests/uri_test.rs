#[cfg(test)]
mod tests {
    use http4r_core::headers::Headers;
    use http4r_core::http_message::Request;
    use http4r_core::uri::Uri;
    use http4r_core::http_message::Method::GET;

    #[test]
    fn can_parse_uri() {
        let uri = Uri::parse("http://authority/some/path?query=string#fragment");
        assert_eq!(uri.scheme, Some("http"));
        assert_eq!(uri.authority, Some("authority"));
        assert_eq!(uri.path, "/some/path");
        assert_eq!(uri.query, Some("query=string"));
        assert_eq!(uri.fragment, Some("fragment"));
    }

    #[test]
    fn supports_relative() {
        let uri = Uri::parse("some/path");
        assert_eq!(uri.scheme, None);
        assert_eq!(uri.authority, None);
        assert_eq!(uri.path, "some/path");
        assert_eq!(uri.query, None);
        assert_eq!(uri.fragment, None);
    }

    #[test]
    fn supports_urns() {
        let uri = Uri::parse("uuid:720f11db-1a29-4a68-a034-43f80b27659d");
        assert_eq!(uri.scheme, Some("uuid"));
        assert_eq!(uri.authority, None);
        assert_eq!(uri.path, "720f11db-1a29-4a68-a034-43f80b27659d");
        assert_eq!(uri.query, None);
        assert_eq!(uri.fragment, None);
    }

    #[test]
    fn is_reverse_able() {
        let original = "http://authority/some/path?query=string#fragment";
        assert_eq!(Uri::parse(original).to_string(), original.to_string());
        let another = "some/path";
        assert_eq!(Uri::parse(another).to_string(), another.to_string());
    }

    #[test]
    fn can_pattern_match_a_request() {
        let request = Request::get(Uri::parse("/some/path"), Headers::empty()).with_header(("Content-Type", "text/plain"));
        match request {
            Request { method: GET, uri: Uri { path: "/some/path", .. }, ref headers, .. }
            if headers.get("Content-Type") == Some("text/plain".to_string()) => {}
            _ => {
                panic!("Should have matched");
            }
        }
    }
}