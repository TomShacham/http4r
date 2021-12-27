
#[cfg(test)]
mod tests {
    use http4r_core::http_message::{add_header, header, js_headers_from_string, js_headers_to_string};

    #[test]
    fn get_header() {
        assert_eq!(header(&vec!(), "foo"), None);
        assert_eq!(header(&vec!(("foo".to_string(), "bar".to_string())), "foo"),
                   Some(("foo".to_string(), "bar".to_string())));
    }

    #[test]
    fn is_case_insensitive_and_preserves_case() {
        assert_eq!(header(&vec!(("fOo".to_string(), "bAr".to_string())), "Foo"),
                   Some(("fOo".to_string(), "bAr".to_string())));
    }

    #[test]
    fn add_headers() {
        let vec1 = vec!(("a".to_string(), "b".to_string()));
        assert_eq!(add_header(&vec1, ("some".to_string(), "other".to_string())),
                   vec!(("a".to_string(), "b".to_string()), ("some".to_string(), "other".to_string())));

        let added = add_header(&vec!(), ("foo".to_string(), "bar".to_string()));
        assert_eq!(added, vec!(("foo".to_string(), "bar".to_string())));

        let added_again = add_header(&added, ("foo".to_string(), "baz".to_string()));
        assert_eq!(added_again, vec!(("foo".to_string(), "bar, baz".to_string())));

        assert_eq!(add_header(&added_again, ("foo".to_string(), "quux".to_string())),
                   vec!(("foo".to_string(), "bar, baz, quux".to_string())));
    }

    #[test]
    fn parse_js_headers_from_string() {
        let headers = js_headers_from_string("Content-Length: 10; Content-Type: text/html");

        assert_eq!(header(&headers, "Content-Length").unwrap(), ("Content-Length".to_string(), "10".to_string()));
        assert_eq!(header(&headers, "Content-Type").unwrap(), ("Content-Type".to_string(), "text/html".to_string()))
    }

    #[test]
    fn parse_js_headers_to_string() {
        let headers = vec!(("Content-Type".to_string(), "text/plain".to_string()), ("Content-Length".to_string(), "10".to_string()));
        let string = js_headers_to_string(&headers);

        assert_eq!(string, "Content-Type: text/plain; Content-Length: 10");
    }
}