use std::net::{TcpListener, TcpStream};
use std::time::Duration;
use rusty::filter::Filter;
use rusty::httpmessage;
use rusty::httpmessage::Method::GET;
use rusty::httpmessage::{Header, HttpMessage, Request, Response};
use rusty::httpmessage::Status::{NotFound, OK};
use rusty::httphandler::HttpHandler;

#[cfg(test)]
mod tests {
    use std::thread;
    use rusty::client::Client;
    use rusty::filter::{IdentityFilter, RawHttpFilter};
    use rusty::httpmessage::get;
    use rusty::router::Router;
    use rusty::server::Server;
    use super::*;

    #[test]
    fn client_over_http() {
        let port = 7878;
        Server::new(Box::new(Router {}), port);
        let client = Client { base_uri: String::from("127.0.0.1"), port };
        let request = get("".to_string(), vec!(), "".to_string());

        assert_eq!("GET / HTTP/1.1\r\n".to_string(), client.handle(request).body);
    }

    #[test]
    fn router_non_http() {
        let router = Router {};
        let request = get("/".to_string(), vec!(), "".to_string());
        let request_to_no_route = get("no/route/here".to_string(), vec!(), "".to_string());

        assert_eq!(OK, router.handle(request).status.into());
        assert_eq!(NotFound, router.handle(request_to_no_route).status);
    }

    #[test]
    fn filter() {
        let filter = RawHttpFilter {
            next: Box::new(IdentityFilter {})
        };

        let router = filter.filter(Box::new(Router {}));

    }
}
//
// fn run_with_server<T>(test: T) -> ()
//     where T: FnOnce() -> () + panic::UnwindSafe
// {
//     let result = panic::catch_unwind(|| {
//         test()
//     });
//     // teardown();
//     assert!(result.is_ok())
// }
