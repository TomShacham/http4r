
#[cfg(test)]
mod tests {
    use http4r_core::headers::Headers;

    #[test]
    fn get_header() {
        assert_eq!(Headers::empty().get("foo"), None);
        assert_eq!(Headers::from(vec!(("foo", "bar"))).get("foo"),
                   Some("bar".to_string()));
    }

    #[test]
    fn is_case_insensitive_and_preserves_case() {
        assert_eq!(Headers::from(vec!(("fOo", "bAr"))).get("Foo"),
                   Some("bAr".to_string()));
    }

    #[test]
    fn add_headers() {
        let headers = Headers::from(vec!(("a", "b")));
        assert_eq!(headers.add_header(("some", "other")).vec,
                   vec!(("a".to_string(), "b".to_string()), ("some".to_string(), "other".to_string())));

        let added = Headers::empty().add_header(("foo", "bar"));
        assert_eq!(added.vec, vec!(("foo".to_string(), "bar".to_string())));

        let added_again = added.add_header(("foo", "baz"));
        assert_eq!(added_again.vec, vec!(("foo".to_string(), "bar, baz".to_string())));

        assert_eq!(added_again.add_header(("foo", "quux")).vec,
                   vec!(("foo".to_string(), "bar, baz, quux".to_string())));
    }

    #[test]
    fn replace_headers() {
        let headers = Headers::from(vec!(("a", "b")));
        let added = headers.add_header(("a", "c"));

        assert_eq!(added.vec, vec!(("a".to_string(), "b, c".to_string())));

        let replaced = added.replace_header(("a", "b"));
        assert_eq!(replaced.vec, vec!(("a".to_string(), "b".to_string())));

        let add_when_using_replace = replaced.replace_header(("new", "value"));
        let with_new_value = vec!(("a".to_string(), "b".to_string()), ("new".to_string(), "value".to_string()));
        assert_eq!(add_when_using_replace.vec, with_new_value)
    }

    #[test]
    fn gives_you_new_headers_when_doing_a_thing_ie_its_immutable(){
        let headers = Headers::from(vec!(("a", "b")));
        let added = headers.add_header(("a", "b"));

        assert_eq!(added.vec, vec!(("a".to_string(), "b, b".to_string())));
        assert_eq!(headers.vec, vec!(("a".to_string(), "b".to_string())));

        let replaced = headers.replace_header(("a", "c"));
        let replace_again = replaced.replace_header(("b", "c"));

        assert_eq!(replace_again.vec, vec!(("a".to_string(), "c".to_string()), ("b".to_string(), "c".to_string())));
        assert_eq!(replaced.vec, vec!(("a".to_string(), "c".to_string())));
    }

    #[test]
    fn from_does_munging(){
        let vec1 = vec!(("a", "b"), ("a", "c"));

        assert_eq!(Headers::from(vec1).vec, vec!(("a".to_string(), "b, c".to_string())))
    }

    #[test]
    fn parse_js_headers_from_string() {
        let headers = Headers::js_headers_from_string("Content-Length: 10; Content-Type: text/html");

        assert_eq!(headers.get("Content-Length").unwrap(), "10".to_string());
        assert_eq!(headers.get("Content-Type").unwrap(), "text/html".to_string())
    }

    #[test]
    fn parse_js_headers_to_string() {
        let headers = vec!(("Content-Type".to_string(), "text/plain".to_string()), ("Content-Length".to_string(), "10".to_string()));
        let string = Headers::js_headers_to_string(&headers);

        assert_eq!(string, "Content-Type: text/plain; Content-Length: 10");
    }
}