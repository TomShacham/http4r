
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
        assert_eq!(Headers::from(vec!(("fOo", "bAr"), ("foo", "BaZ"))).get("Foo"),
                   Some("bAr, BaZ".to_string()));
    }

    #[test]
    fn add_headers() {
        let headers = Headers::from(vec!(("a", "b")));
        assert_eq!(headers.add(("some", "other")).vec,
                   vec!(("a".to_string(), "b".to_string()), ("some".to_string(), "other".to_string())));

        let added = Headers::empty().add(("foo", "bar"));
        assert_eq!(added.vec, vec!(("foo".to_string(), "bar".to_string())));

        let added_again = added.add(("foo", "baz"));
        assert_eq!(added_again.vec, vec!(("foo".to_string(), "bar, baz".to_string())));

        // case is different
        let case_insensitive = added_again.add(("Foo", "quux"));
        assert_eq!(case_insensitive.vec, vec!(("foo".to_string(), "bar, baz, quux".to_string())));
    }

    #[test]
    fn add_many_headers() {
        let headers = Headers::from(vec!(("a", "b")));
        let adding = Headers::from(vec!(("some", "other"), ("and", "more")));
        assert_eq!(headers.add_all(adding).vec,
                   vec!(
                        ("some".to_string(), "other".to_string()),
                        ("and".to_string(), "more".to_string()),
                        ("a".to_string(), "b".to_string()),
                   ));

    }

    #[test]
    fn replace_headers() {
        let headers = Headers::from(vec!(("a", "b")));
        let added = headers.add(("a", "c"));

        assert_eq!(added.vec, vec!(("a".to_string(), "b, c".to_string())));

        let replaced = added.replace(("a", "b"));
        assert_eq!(replaced.vec, vec!(("a".to_string(), "b".to_string())));

        let add_when_using_replace = replaced.replace(("new", "value"));
        let with_new_value = vec!(("a".to_string(), "b".to_string()), ("new".to_string(), "value".to_string()));
        assert_eq!(add_when_using_replace.vec, with_new_value);

        let case_insensitive = add_when_using_replace.replace(("NEW", "VALUE"));
        let with_newer_value = vec!(("a".to_string(), "b".to_string()),
                                    ("new".to_string(), "VALUE".to_string()));
        assert_eq!(case_insensitive.vec, with_newer_value);
    }

    #[test]
    fn remove_headers() {
        let headers = Headers::from(vec!(("a", "b")));
        assert_eq!(headers.remove("a").vec, vec!());

        let multi = Headers::from(vec!(("a", "b"), ("b", "c"), ("b", "d")));
        assert_eq!(headers.remove("b").vec, vec!(("a".to_string(), "b".to_string())));

        let case_insensitive = multi.remove("A");
        assert_eq!(case_insensitive.vec, vec!(("b".to_string(), "c, d".to_string())))
    }

    #[test]
    fn gives_you_new_headers_when_doing_a_thing_ie_its_immutable(){
        let headers = Headers::from(vec!(("a", "b")));
        let added = headers.add(("a", "b"));

        assert_eq!(added.vec, vec!(("a".to_string(), "b, b".to_string())));
        assert_eq!(headers.vec, vec!(("a".to_string(), "b".to_string())));

        let replaced = headers.replace(("a", "c"));
        let replace_again = replaced.replace(("b", "c"));

        assert_eq!(replace_again.vec, vec!(("a".to_string(), "c".to_string()), ("b".to_string(), "c".to_string())));
        assert_eq!(replaced.vec, vec!(("a".to_string(), "c".to_string())));

        let removed = headers.remove("b");
        let removed_again = removed.remove("a");

        assert_eq!(removed_again.vec, vec!());
        assert_eq!(removed.vec, vec!(("a".to_string(), "b".to_string())));
    }

    #[test]
    fn filter(){
        let vec = vec!(("a", "b"));
        assert_eq!(Headers::from(vec).filter(vec!()).vec, vec!());

        let vec2 = vec!(("a", "b"), ("b", "c"));
        assert_eq!(Headers::from(vec2).filter(vec!("a")).vec,
                   vec!(("a".to_string(), "b".to_string())));

        let vec3 = vec!(("a", "b"), ("b", "c"), ("c", "d"));
        assert_eq!(Headers::from(vec3).filter(vec!("a", "c")).vec,
                   vec!(
                       ("a".to_string(), "b".to_string()),
                       ("c".to_string(), "d".to_string()),
                   ));
    }
    
    #[test]
    fn is_empty() {
        assert!(Headers::from(vec!()).is_empty());

        assert_eq!(Headers::from(vec!(("a", "b"))).is_empty(), false);
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