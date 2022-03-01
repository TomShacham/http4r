#[cfg(test)]
mod tests {
    use http4r_core::client::Client;
    use http4r_core::handler::Handler;
    use http4r_core::headers::Headers;
    use http4r_core::http_message::Request;
    use http4r_core::http_message::Status::NotFound;
    use http4r_core::server::Server;
    use http4r_core::uri::Uri;
    use http4r_example_app::ok_handler::OkHandler;

    #[test]
    fn not_found() {
        let mut server = Server::new(0);
        // start listening on next available port, close on finish = true
        server.start(|| { Ok(OkHandler) }, true);

        // request to some url we do not serve
        let request = Request::get(Uri::parse("/not-found"), Headers::empty());

        let mut client = Client::new("127.0.0.1", server.port, None);
        // make an http request to our server and assert response status is Not Found.
        client.handle(request, |res| {
            assert_eq!(res.status, NotFound);
        })
    }
}