use std::env;
use http4r_core::client::Client;
use http4r_core::handler::Handler;
use http4r_core::headers::Headers;
use http4r_core::http_message::{body_string, Request};
use http4r_core::http_message::Body::{BodyStream, BodyString};
use http4r_core::server::Server;
use http4r_core::uri::Uri;
use http4r_example_app::app::App;
use http4r_example_app::environment::Environment;

fn main() {
    let env = Environment::from(env::vars())
        .with(vec!(("ENV", "local")));

    let port = env.get("PORT").map(|p| p.parse::<u16>().unwrap_or(0));
    let mut server = Server::new(port.unwrap_or(0));

    let mut client = Client::new("uk.yahoo.com", 80, None);
    let request = Request::get(
        Uri::parse("/"),
        Headers::from(vec!(
            ("accept", "application/json"),
            ("host", "uk.yahoo.com"),
        ))
    );
    client.handle(request, |res| {
        println!("hello");
        println!("st {}", res.status.to_string());
        println!("{}", body_string(res.body));
    });

    server.start(move || {
        Ok(App::production(env.copy()))
    }, false);
}


