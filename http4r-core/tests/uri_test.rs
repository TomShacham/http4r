#[cfg(test)]
mod tests {
    use http4r_core::headers::Headers;
    use http4r_core::http_message::Request;
    use http4r_core::uri::Uri;
    use http4r_core::http_message::Method::GET;

    #[test]
    fn can_parse_uri() {
        let uri = Uri::parse("http://authority/some/path?query=string/with/slashes/and?question?marks#fragment");
        assert_eq!(uri.scheme, Some("http"));
        assert_eq!(uri.authority, Some("authority"));
        assert_eq!(uri.path, "/some/path");
        assert_eq!(uri.query, Some("query=string/with/slashes/and?question?marks"));
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

    #[test]
    fn can_replace_bits_and_doesnt_mutate() {
        let uri = Uri::parse("/");
        assert_eq!(uri.with_scheme("https").to_string(), "https:/");
        assert_eq!(uri.to_string(), "/"); // not mutated by above
        assert_eq!(Uri::parse("/").with_path("/new/path").to_string(), "/new/path");
        assert_eq!(Uri::parse("/").with_authority("user@password").to_string(), "//user@password/");
        assert_eq!(Uri::parse("/").with_query("foo=bar&baz=quux").to_string(), "/?foo=bar&baz=quux");
        assert_eq!(Uri::parse("/").with_fragment("frag").to_string(), "/#frag");

        assert_eq!(Uri::parse("/")
                       .with_path("/new/path")
                       .with_authority("user@password")
                       .with_scheme("https")
                       .with_query("foo=bar&baz=quux")
                       .with_fragment("frag").to_string(), "https://user@password/new/path?foo=bar&baz=quux#frag");
    }
}