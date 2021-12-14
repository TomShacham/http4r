use rusty::httpmessage::Status::{NotFound, OK};

#[cfg(test)]
mod tests {
    use rusty::client::Client;
    use rusty::httphandler::HttpHandler;
    use rusty::httpmessage::{body_string, get, ok, Request};
    use rusty::httpmessage::Body::BodyString;
    use rusty::router::Router;
    use rusty::server::{read_to_buffer, Server};
    use super::*;

    #[test]
    fn client_over_http() {
        let port = 7878;
        let handler: HttpHandler = {
            |_req: Request| {
                return ok(vec!(("hi".to_string(), "there".to_string())), BodyString("response body".to_string()));
            }
        };

        Server::new(handler, port, None);
        let client = Client { base_uri: String::from("127.0.0.1"), port };
        let request = get("".to_string(), vec!());
        let response = client.handle(request);

        assert_eq!("OK", response.status.to_string());
        assert_eq!("response body".to_string(), body_string(response.body));
        assert_eq!("hi: there", response.headers.iter().map(|(name, value)| format!("{}: {}", name, value)).collect::<String>());
    }

    #[test]
    fn router_non_http() {
        let router = Router {};
        let request = get("/".to_string(), vec!());
        let request_to_no_route = get("no/route/here".to_string(), vec!());

        assert_eq!(OK, router.handle(request).status.into());
        assert_eq!(NotFound, router.handle(request_to_no_route).status);
    }
}