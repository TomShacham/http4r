
#[cfg(test)]
mod tests {
    use rusty::http_message::{add_header, header};

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
}