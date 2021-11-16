#[cfg(test)]
mod tests {
    use rusty::httpmessage::Method::{GET, POST};
    use rusty::httpmessage::Request;

    #[test]
    fn request_from_str() {
        let str = "POST /foo/bar HTTP/1.1\r\nConnection: close\r\nAccept: text/html\r\n\r\nbody";
        let request = Request::from(str);

        assert_eq!(request.method, POST);
        assert_eq!(request.headers.first().unwrap().clone(), ("Connection".to_string(), "close".to_string()));
        assert_eq!(request.headers.last().unwrap().clone(), ("Accept".to_string(), "text/html".to_string()));
        assert_eq!(request.body, "body".to_string());
        assert_eq!(request.uri, "/foo/bar".to_string());
    }

    #[test]
    fn request_from_str_no_body() {
        let str = "GET /foo/bar HTTP/1.1\r\nConnection: close\r\nAccept: text/html\r\n\r\n";
        let request = Request::from(str);

        assert_eq!(request.method, GET);
        assert_eq!(request.headers.first().unwrap().clone(), ("Connection".to_string(), "close".to_string()));
        assert_eq!(request.headers.last().unwrap().clone(), ("Accept".to_string(), "text/html".to_string()));
        assert_eq!(request.body, "".to_string());
        assert_eq!(request.uri, "/foo/bar".to_string());
    }

    #[test]
    fn request_from_str_no_headers() {
        let str = "GET /foo/bar HTTP/1.1\r\n\r\n";
        let request = Request::from(str);

        assert_eq!(request.method, GET);
        assert_eq!(request.headers.first(), None);
        assert_eq!(request.body, "".to_string());
        assert_eq!(request.uri, "/foo/bar".to_string());
    }


}