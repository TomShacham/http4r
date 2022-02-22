use std::env;
use std::option::Option::None;
use http4r_core::server::Server;
use http4r_example_app::app::App;
use http4r_example_app::environment::Environment;

fn main() {
    let env = Environment::from(env::vars())
        .with(vec!(("ENV", "local")));

    let port = env.get("PORT").map(|p| p.parse::<u16>().unwrap_or(0));
    let mut server = Server::new(port.unwrap_or(0));

    server.start(move || {
        Ok(App::production(env.copy()))
    });
}


