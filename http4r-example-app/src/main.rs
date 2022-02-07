use std::env;
use http4r_core::server::Server;
use http4r_example_app::app::App;
use http4r_example_app::environment::Environment;

fn main() {
    let env = Environment::from(env::vars());
    let port = env.get("PORT".to_string()).map(|p| p.1.parse().unwrap_or(0));
    let mut server = Server::new(port.unwrap_or(0));

    server.start(|| { Ok(App::production()) })
}


