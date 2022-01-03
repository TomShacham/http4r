#[cfg(test)]
mod tests {
    use http4r_core::query::Query;

    #[test]
    fn get() {
        assert_eq!(Query::empty().get("foo"), None);
        assert_eq!(Query::from(vec!(("foo", "bar"))).get("foo"), Some("bar".to_string()));
    }

    #[test]
    fn is_case_sensitive_and_preserves_case() {
        assert_eq!(Query::from(vec!(("fOo", "bAr"))).get("Foo"), None);
        assert_eq!(Query::from(vec!(("fOo", "bAr"))).get("fOo"), Some("bAr".to_string()));
    }

    #[test]
    fn add() {
        let query = Query::from(vec!(("a", "b")));
        assert_eq!(query.add(("some", "other")).vec,
                   vec!(("a".to_string(), "b".to_string()), ("some".to_string(), "other".to_string())));

        let added = Query::empty().add(("foo", "bar"));
        assert_eq!(added.vec, vec!(("foo".to_string(), "bar".to_string())));

        let added_again = added.add(("foo", "baz"));
        assert_eq!(added_again.vec,
                   vec!(("foo".to_string(), "bar".to_string()), ("foo".to_string(), "baz".to_string())));

        assert_eq!(added_again.add(("foo", "quux")).vec,
                   vec!(("foo".to_string(), "bar".to_string()),
                        ("foo".to_string(), "baz".to_string()),
                        ("foo".to_string(), "quux".to_string()),
                   ));
    }

    #[test]
    fn replace() {
        let query = Query::from(vec!(("a", "b")));
        let added = query.add(("a", "c"));

        assert_eq!(added.vec, vec!(
            ("a".to_string(), "b".to_string()),
            ("a".to_string(), "c".to_string()),
        ));

        let replaced = added.replace(("a", "b"));
        assert_eq!(replaced.vec, vec!(("a".to_string(), "b".to_string())));

        let add_by_using_replace = replaced.replace(("new", "value"));
        assert_eq!(add_by_using_replace.vec, vec!(
            ("a".to_string(), "b".to_string()),
            ("new".to_string(), "value".to_string())))
    }

    #[test]
    fn gives_you_new_query_when_doing_a_thing_ie_its_immutable() {
        let query = Query::from(vec!(("a", "b")));
        let added = query.add(("a", "b"));

        assert_eq!(added.vec, vec!(("a".to_string(), "b".to_string()), ("a".to_string(), "b".to_string())));
        assert_eq!(query.vec, vec!(("a".to_string(), "b".to_string())));

        let replaced = query.replace(("a", "c"));
        let replace_again = replaced.replace(("b", "c"));

        assert_eq!(query.vec, vec!(("a".to_string(), "b".to_string())));
        assert_eq!(replaced.vec, vec!(("a".to_string(), "c".to_string())));
        assert_eq!(replace_again.vec, vec!(("a".to_string(), "c".to_string()),
                                           ("b".to_string(), "c".to_string())));
    }

    // #[test]
    // fn from_does_munging(){
    //     let vec1 = vec!(("a", "b"), ("a", "c"));
    //
    //     assert_eq!(Headers::from(vec1).vec, vec!(("a".to_string(), "b, c".to_string())))
    // }
    //
    // #[test]
    // fn parse_js_headers_from_string() {
    //     let headers = Headers::js_headers_from_string("Content-Length: 10; Content-Type: text/html");
    //
    //     assert_eq!(headers.get("Content-Length").unwrap(), "10".to_string());
    //     assert_eq!(headers.get("Content-Type").unwrap(), "text/html".to_string())
    // }
    //
    // #[test]
    // fn parse_js_headers_to_string() {
    //     let headers = vec!(("Content-Type".to_string(), "text/plain".to_string()), ("Content-Length".to_string(), "10".to_string()));
    //     let string = Headers::js_headers_to_string(&headers);
    //
    //     assert_eq!(string, "Content-Type: text/plain; Content-Length: 10");
    // }
}