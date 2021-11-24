use rusty::httpmessage::Status::{NotFound, OK};

#[cfg(test)]
mod tests {
    use rusty::client::Client;
    use rusty::httpmessage::{get, ok, Request};
    use rusty::router::Router;
    use rusty::server::Server;
    use super::*;

    #[test]
    fn client_over_http() {
        let port = 7878;
        let handler = {
            |_req: Request| {
                ok(vec!(), "response body".to_string())
            }
        };

        Server::new(handler, port, None);
        let client = Client { base_uri: String::from("127.0.0.1"), port };
        let request = get("".to_string(), vec!());
        let response = client.handle(request);

        assert_eq!("OK", response.status.to_string());
        assert_eq!("Content-Length: 13", response.headers.iter().map(|(name, value)| format!("{}: {}", name, value)).collect::<String>());
        assert_eq!("response body", response.body.to_string());
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